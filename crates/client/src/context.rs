use crate::{
    errors_pool::ErrorPoolExt,
    states::States,
    subscriber::EguiLayer,
    views::{self, profiles::ProfilesPage, settings::SettingsPage, Logs, ModManager, ModManagerState, ProfileInfo, View},
    Tab, TabKind,
};
use eframe::egui::{self};
use egui_dock::TabViewer;
use egui_file_dialog::FileDialog;
use egui_task_manager::TaskManager;
use nomi_core::{
    repository::launcher_manifest::{Latest, LauncherManifest},
    state::get_launcher_manifest,
};

pub struct MyContext {
    pub egui_layer: EguiLayer,
    pub launcher_manifest: &'static LauncherManifest,
    pub file_dialog: FileDialog,

    pub manager: TaskManager,
    pub states: States,

    pub is_allowed_to_take_action: bool,
    pub is_profile_window_open: bool,

    pub images_clean_requested: bool,
}

impl MyContext {
    pub fn new(egui_layer: EguiLayer) -> Self {
        const EMPTY_MANIFEST: &LauncherManifest = &LauncherManifest {
            latest: Latest {
                release: String::new(),
                snapshot: String::new(),
            },
            versions: Vec::new(),
        };

        let launcher_manifest_ref = pollster::block_on(get_launcher_manifest()).report_error().unwrap_or(EMPTY_MANIFEST);

        Self {
            egui_layer,
            launcher_manifest: launcher_manifest_ref,
            file_dialog: FileDialog::new(),
            is_profile_window_open: false,

            states: States::new(),
            manager: TaskManager::new(),
            is_allowed_to_take_action: true,
            images_clean_requested: false,
        }
    }

    pub fn request_images_clean(&mut self) {
        self.images_clean_requested = true
    }
}

impl TabViewer for MyContext {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab.kind.id()).into()
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        match &tab.kind {
            TabKind::Mods { profile } => {
                let profile = profile.read();
                self.states.profiles.profiles.find_profile(profile.profile.id).is_none()
            }
            TabKind::ProfileInfo { profile } => {
                let profile = profile.read();
                self.states.profiles.profiles.find_profile(profile.profile.id).is_none()
            }
            _ => false,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match &tab.kind {
            TabKind::Profiles => ProfilesPage {
                is_allowed_to_take_action: self.is_allowed_to_take_action,
                profile_info_state: &mut self.states.profile_info,
                manager: &mut self.manager,
                settings_state: &self.states.settings,
                profiles_state: &mut self.states.profiles,
                menu_state: &mut self.states.add_profile_menu_state,
                tabs_state: &mut self.states.tabs,
                logs_state: &self.states.logs_state,

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
            TabKind::Logs => Logs {
                egui_layer: &self.egui_layer,
                logs_state: &mut self.states.logs_state,
            }
            .ui(ui),
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
            TabKind::ProfileInfo { profile } => ProfileInfo {
                profiles: &self.states.profiles.profiles,
                task_manager: &mut self.manager,
                profile: profile.clone(),
                tabs_state: &mut self.states.tabs,
                profile_info_state: &mut self.states.profile_info,
            }
            .ui(ui),
        };
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        if let TabKind::Mods { profile: _ } = tab.kind {
            self.states.mod_manager = ModManagerState::new();
            self.request_images_clean()
        }

        self.states.tabs.0.remove(&tab.id);
        true
    }
}
