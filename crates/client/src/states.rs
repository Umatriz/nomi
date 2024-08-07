use std::{collections::HashSet, path::PathBuf};

use egui_task_manager::{Caller, Task, TaskManager};
use nomi_core::{
    downloads::{java::JavaDownloader, progress::MappedSender, traits::Downloader},
    fs::read_toml_config_sync,
    DOT_NOMI_JAVA_DIR, DOT_NOMI_JAVA_EXECUTABLE, DOT_NOMI_PROFILES_CONFIG, DOT_NOMI_SETTINGS_CONFIG,
};
use tracing::info;

use crate::{
    collections::JavaCollection,
    errors_pool::{ErrorPoolExt, ErrorsPoolState},
    views::{
        add_tab_menu::TabsState,
        profiles::ProfilesState,
        settings::{ClientSettingsState, SettingsState},
        AddProfileMenuState, LogsState, ModManagerState, ProfileInfoState, ProfilesConfig,
    },
};

pub struct States {
    pub tabs: TabsState,
    pub errors_pool: ErrorsPoolState,

    pub logs_state: LogsState,
    pub java: JavaState,
    pub profiles: ProfilesState,
    pub settings: SettingsState,
    pub client_settings: ClientSettingsState,
    pub add_profile_menu_state: AddProfileMenuState,
    pub mod_manager: ModManagerState,
    pub profile_info: ProfileInfoState,
}

impl Default for States {
    fn default() -> Self {
        let settings = read_toml_config_sync::<SettingsState>(DOT_NOMI_SETTINGS_CONFIG).unwrap_or_default();

        Self {
            tabs: TabsState::new(),
            errors_pool: ErrorsPoolState::default(),
            logs_state: LogsState::new(),
            java: JavaState::new(),
            profiles: ProfilesState {
                currently_downloading_profiles: HashSet::new(),
                profiles: read_toml_config_sync::<ProfilesConfig>(DOT_NOMI_PROFILES_CONFIG).unwrap_or_default(),
            },
            client_settings: settings.client_settings.clone(),
            settings,
            add_profile_menu_state: AddProfileMenuState::default(),
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

    pub fn download_java(&mut self, manager: &mut TaskManager) {
        info!("Downloading Java");

        self.is_downloaded = true;

        let caller = Caller::progressing(|progress| async move {
            let downloader = JavaDownloader::new(PathBuf::from(DOT_NOMI_JAVA_DIR));

            let _ = progress.set_total(downloader.total());

            let io = downloader.io();

            let mapped_sender = MappedSender::new_progress_mapper(Box::new(progress.sender()));

            Box::new(downloader).download(&mapped_sender).await;

            io.await.report_error();
        });

        let task = Task::new("Java downloading", caller);
        manager.push_task::<JavaCollection>(task);
    }
}
