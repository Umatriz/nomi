use crate::{
    channel::Channel,
    components::{
        download_progress::DownloadProgress, profiles::ProfilesPage, settings::SettingsPage,
        Component,
    },
    states::States,
    Tab, TabKind,
};
use eframe::egui::{self, ScrollArea};
use egui_dock::TabViewer;
use egui_file_dialog::FileDialog;
use egui_tracing::EventCollector;
use nomi_core::{
    configs::profile::VersionProfile, downloads::traits::DownloadResult,
    repository::launcher_manifest::LauncherManifest, state::get_launcher_manifest,
};

pub struct MyContext {
    pub collector: EventCollector,
    pub launcher_manifest: &'static LauncherManifest,
    pub file_dialog: FileDialog,

    pub states: States,

    pub is_profile_window_open: bool,

    pub download_result_channel: Channel<VersionProfile>,
    pub download_progress_channel: Channel<DownloadResult>,
    pub download_total_channel: Channel<u32>,
}

impl MyContext {
    pub fn new(collector: EventCollector) -> Self {
        let launcher_manifest_ref = pollster::block_on(get_launcher_manifest()).unwrap();

        Self {
            collector,
            launcher_manifest: launcher_manifest_ref,
            file_dialog: FileDialog::new(),
            is_profile_window_open: false,
            download_result_channel: Channel::new(100),
            download_progress_channel: Channel::new(500),
            download_total_channel: Channel::new(100),

            states: States::new(),
        }
    }
}

impl TabViewer for MyContext {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.kind().name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match &mut tab.kind_mut() {
            TabKind::Profiles { menu_state } => ProfilesPage {
                download_result_tx: self.download_result_channel.clone_tx(),
                download_progress_tx: self.download_progress_channel.clone_tx(),
                download_total_tx: self.download_total_channel.clone_tx(),

                state: &mut self.states.profiles,
                menu_state,

                launcher_manifest: self.launcher_manifest,
                is_profile_window_open: &mut self.is_profile_window_open,
            }
            .ui(ui),
            TabKind::Settings => SettingsPage {
                settings_state: &mut self.states.settings,
                client_settings_state: &mut self.states.client_settings,
                file_dialog: &mut self.file_dialog,
            }
            .ui(ui),
            TabKind::Logs => {
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.add(egui_tracing::Logs::new(self.collector.clone()));
                });
            }
            TabKind::DownloadProgress => {
                DownloadProgress {
                    download_progress_state: &mut self.states.download_progress,
                    profiles_state: &mut self.states.profiles,

                    download_result_rx: &mut self.download_result_channel,
                    download_progress_rx: &mut self.download_progress_channel,
                    download_total_rx: &mut self.download_total_channel,
                }
                .ui(ui);
            }
        };
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.states.tabs.0.remove(tab.id());
        true
    }
}
