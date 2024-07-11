use std::ops::Deref;

use crate::{
    errors_pool::ErrorPoolExt,
    states::States,
    views::{self, profiles::ProfilesPage, settings::SettingsPage, ModManager, ProfileInfo, View},
    TabKind,
};
use eframe::egui::{self, ScrollArea};
use egui_dock::TabViewer;
use egui_file_dialog::FileDialog;
use egui_infinite_scroll::InfiniteScroll;
use egui_task_manager::TaskManager;
use egui_tracing::EventCollector;
use nomi_core::{
    repository::launcher_manifest::{Latest, LauncherManifest},
    state::get_launcher_manifest,
};
use nomi_modding::{
    modrinth::search::{Hit, SearchData},
    Query,
};

pub struct MyContext {
    pub collector: EventCollector,
    pub launcher_manifest: &'static LauncherManifest,
    pub file_dialog: FileDialog,

    pub manager: TaskManager,
    pub states: States,

    pub is_allowed_to_take_action: bool,
    pub is_profile_window_open: bool,
}

impl MyContext {
    pub fn new(collector: EventCollector) -> Self {
        const EMPTY_MANIFEST: &LauncherManifest = &LauncherManifest {
            latest: Latest {
                release: String::new(),
                snapshot: String::new(),
            },
            versions: Vec::new(),
        };

        let launcher_manifest_ref = pollster::block_on(get_launcher_manifest())
            .report_error()
            .unwrap_or(EMPTY_MANIFEST);

        Self {
            collector,
            launcher_manifest: launcher_manifest_ref,
            file_dialog: FileDialog::new(),
            is_profile_window_open: false,

            states: States::new(),
            manager: TaskManager::new(),
            is_allowed_to_take_action: true,
        }
    }
}

impl TabViewer for MyContext {
    type Tab = TabKind;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            TabKind::Profiles => ProfilesPage {
                is_allowed_to_take_action: self.is_allowed_to_take_action,
                manager: &mut self.manager,
                settings_state: &self.states.settings,
                profiles_state: &mut self.states.profiles,
                menu_state: &mut self.states.add_profile_menu_state,
                tabs_state: &mut self.states.tabs,

                launcher_manifest: self.launcher_manifest,
                is_profile_window_open: &mut self.is_profile_window_open,
            }
            .ui(ui),
            TabKind::Settings => SettingsPage {
                java_state: &mut self.states.java,
                manager: &mut self.manager,
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
                views::DownloadingProgress {
                    manager: &self.manager,
                    profiles_state: &mut self.states.profiles,
                }
                .ui(ui);
            }
            TabKind::Mods { profile } => ModManager {
                task_manager: &mut self.manager,
                profiles_config: &mut self.states.profiles.profiles,
                mod_manager_state: &mut self.states.mod_manager,
                profile: profile.clone(),
            }
            .ui(ui),
            TabKind::ProfileInfo { profile } => {
                ProfileInfo {
                    profile: profile.clone(),
                    tabs_state: &mut self.states.tabs,
                    profile_info_state: &mut self.states.profile_info,
                }
            }
            .ui(ui),
        };
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.states.tabs.0.remove(tab);
        true
    }
}
