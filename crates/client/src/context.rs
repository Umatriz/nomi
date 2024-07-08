use crate::{
    errors_pool::ErrorPoolExt,
    states::States,
    views::{self, profiles::ProfilesPage, settings::SettingsPage, View},
    Tab, TabKind,
};
use eframe::egui::{self, ScrollArea};
use egui_dock::TabViewer;
use egui_file_dialog::FileDialog;
use egui_task_manager::TaskManager;
use egui_tracing::EventCollector;
use nomi_core::{
    repository::launcher_manifest::{Latest, LauncherManifest},
    state::get_launcher_manifest,
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

        // let mut manager = TasksManager::new();

        // manager.add_collection::<AssetsCollection>();

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
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.kind().name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match &mut tab.kind_mut() {
            TabKind::Profiles => ProfilesPage {
                is_allowed_to_take_action: self.is_allowed_to_take_action,
                manager: &mut self.manager,
                settings_state: &self.states.settings,
                profiles_state: &mut self.states.profiles,
                menu_state: &mut self.states.add_profile_menu_state,

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
        };
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.states.tabs.0.remove(tab.id());
        true
    }
}
