use std::path::PathBuf;

use eframe::egui::{Context, Ui};
use egui_task_manager::{Caller, Task, TaskManager};
use nomi_core::{
    downloads::{java::JavaDownloader, progress::MappedSender, traits::Downloader},
    fs::read_toml_config_sync,
    repository::java_runner::JavaRunner,
    DOT_NOMI_JAVA_DIR, DOT_NOMI_JAVA_EXECUTABLE, DOT_NOMI_SETTINGS_CONFIG,
};
use tracing::info;

use crate::{
    collections::JavaDownloadingCollection,
    errors_pool::ErrorPoolExt,
    views::{
        add_tab_menu::TabsState,
        profiles::InstancesState,
        settings::{ClientSettingsState, SettingsState},
        AddProfileMenuState, CreateInstanceMenuState, LogsState, ModManagerState, ProfileInfoState,
    },
};

pub struct States {
    pub tabs: TabsState,

    pub logs_state: LogsState,
    pub java: JavaState,
    pub instances: InstancesState,
    pub settings: SettingsState,
    pub client_settings: ClientSettingsState,
    pub add_profile_menu: AddProfileMenuState,
    pub create_instance_menu: CreateInstanceMenuState,
    pub mod_manager: ModManagerState,
    pub profile_info: ProfileInfoState,
}

impl Default for States {
    fn default() -> Self {
        let settings = read_toml_config_sync::<SettingsState>(DOT_NOMI_SETTINGS_CONFIG).unwrap_or_default();

        Self {
            tabs: TabsState::new(),
            logs_state: LogsState::new(),
            java: JavaState::new(),
            instances: InstancesState::new(),
            client_settings: settings.client_settings.clone(),
            settings,
            add_profile_menu: AddProfileMenuState::new(),
            create_instance_menu: CreateInstanceMenuState::new(),
            mod_manager: ModManagerState::new(),
            profile_info: ProfileInfoState::new(),
        }
    }
}

impl States {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
pub struct JavaState {
    pub is_downloaded: bool,
}

impl JavaState {
    pub fn new() -> Self {
        let res = std::process::Command::new("java").arg("--version").spawn();
        Self {
            is_downloaded: res.is_ok() || PathBuf::from(DOT_NOMI_JAVA_EXECUTABLE).exists(),
        }
    }

    pub fn download_java(&mut self, manager: &mut TaskManager, ctx: Context) {
        info!("Downloading Java");

        self.is_downloaded = true;

        let caller = Caller::progressing(|progress| async move {
            let downloader = JavaDownloader::new(PathBuf::from(DOT_NOMI_JAVA_DIR));

            let _ = progress.set_total(downloader.total());

            let io = downloader.io();

            let mapped_sender = MappedSender::new_progress_mapper(Box::new(progress.sender())).with_side_effect(move || ctx.request_repaint());

            Box::new(downloader).download(&mapped_sender).await;

            io.await.report_error();
        });

        let task = Task::new("Java downloading", caller);
        manager.push_task::<JavaDownloadingCollection>(task);
    }
}

pub fn download_java_and_update_config(ui: &mut Ui, manager: &mut TaskManager, java_state: &mut JavaState, settings_state: &mut SettingsState) {
    java_state.download_java(manager, ui.ctx().clone());
    settings_state.java = JavaRunner::path(PathBuf::from(DOT_NOMI_JAVA_EXECUTABLE));
    settings_state.update_config();
}
