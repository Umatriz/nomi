use std::{collections::HashSet, path::PathBuf, sync::Arc};

use eframe::egui::{self, popup_below_widget, Align2, Button, Id, PopupCloseBehavior, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use egui_task_manager::{Caller, Task, TaskManager};
use nomi_core::{
    configs::profile::{Loader, ProfileState, VersionProfile},
    fs::{read_toml_config, read_toml_config_sync, write_toml_config, write_toml_config_sync},
    instance::launch::arguments::UserData,
    repository::{launcher_manifest::LauncherManifest, username::Username},
    DOT_NOMI_PROFILES_CONFIG,
};
use serde::{Deserialize, Serialize};

use crate::{
    collections::{AssetsCollection, GameDeletionCollection, GameDownloadingCollection, GameRunnerCollection},
    download::{task_assets, task_download_version},
    errors_pool::ErrorPoolExt,
    TabKind,
};

use super::{
    add_profile_menu::{AddProfileMenu, AddProfileMenuState},
    load_mods,
    settings::SettingsState,
    ModsConfig, TabsState, View,
};

pub struct ProfilesPage<'a> {
    pub is_allowed_to_take_action: bool,
    pub manager: &'a mut TaskManager,
    pub settings_state: &'a SettingsState,

    pub is_profile_window_open: &'a mut bool,

    pub tabs_state: &'a mut TabsState,
    pub profiles_state: &'a mut ProfilesState,
    pub menu_state: &'a mut AddProfileMenuState,

    pub launcher_manifest: &'static LauncherManifest,
}

pub struct ProfilesState {
    pub currently_downloading_profiles: HashSet<usize>,
    pub profiles: ProfilesConfig,
}

#[derive(Default, PartialEq, Eq, Hash, Clone)]
pub struct SimpleProfile {
    pub name: String,
    pub version: String,
    pub loader: Loader,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ProfilesConfig {
    pub profiles: Vec<Arc<ModdedProfile>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ModdedProfile {
    pub profile: VersionProfile,
    pub mods: ModsConfig,
}

impl ModdedProfile {
    pub fn new(profile: VersionProfile) -> Self {
        Self {
            profile,
            mods: ModsConfig::default(),
        }
    }
}

impl ProfilesConfig {
    pub fn find_profile(&self, target_id: usize) -> Option<&Arc<ModdedProfile>> {
        self.profiles.iter().find(|p| p.profile.id == target_id)
    }

    pub fn read() -> Self {
        read_toml_config_sync::<ProfilesConfig>(DOT_NOMI_PROFILES_CONFIG).unwrap_or_default()
    }

    pub async fn read_async() -> Self {
        read_toml_config::<ProfilesConfig>(DOT_NOMI_PROFILES_CONFIG).await.unwrap_or_default()
    }

    pub fn try_read() -> anyhow::Result<Self> {
        read_toml_config_sync::<ProfilesConfig>(DOT_NOMI_PROFILES_CONFIG)
    }

    pub fn add_profile(&mut self, profile: ModdedProfile) {
        self.profiles.push(profile.into())
    }

    pub fn create_id(&self) -> usize {
        match &self.profiles.iter().max_by_key(|profile| profile.profile.id) {
            Some(v) => v.profile.id + 1,
            None => 0,
        }
    }

    pub fn update_config(&self) -> anyhow::Result<()> {
        write_toml_config_sync(&self, DOT_NOMI_PROFILES_CONFIG)
    }

    pub async fn update_config_async(&self) -> anyhow::Result<()> {
        write_toml_config(&self, DOT_NOMI_PROFILES_CONFIG).await
    }
}

impl View for ProfilesPage<'_> {
    fn ui(self, ui: &mut Ui) {
        {
            ui.toggle_value(self.is_profile_window_open, "Add new profile");

            egui::Window::new("Create new profile")
                .title_bar(true)
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .movable(false)
                .open(self.is_profile_window_open)
                .show(ui.ctx(), |ui| {
                    AddProfileMenu {
                        menu_state: self.menu_state,
                        profiles_state: self.profiles_state,
                        launcher_manifest: self.launcher_manifest,
                        manager: self.manager, // is_profile_window_open: self.is_profile_window_open,
                    }
                    .ui(ui);
                });
        }

        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

        TableBuilder::new(ui)
            .column(Column::auto().at_least(120.0).at_most(240.0))
            .columns(Column::auto(), 5)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("Name");
                });
                header.col(|ui| {
                    ui.label("Version");
                });
                header.col(|ui| {
                    ui.label("Loader");
                });
            })
            .body(|mut body| {
                let mut is_deleting = vec![];

                for (index, profile) in self.profiles_state.profiles.profiles.iter().enumerate() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Label::new(&profile.profile.name).truncate());
                        });
                        row.col(|ui| {
                            ui.label(profile.profile.version());
                        });
                        row.col(|ui| {
                            ui.label(profile.profile.loader_name());
                        });
                        row.col(|ui| match &profile.profile.state {
                            ProfileState::Downloaded(instance) => {
                                if ui.add_enabled(self.is_allowed_to_take_action, egui::Button::new("Launch")).clicked() {
                                    let user_data = UserData {
                                        username: Username::new(self.settings_state.username.clone()).unwrap(),
                                        uuid: Some(self.settings_state.uuid.clone()),
                                        access_token: None,
                                    };

                                    let instance = instance.clone();
                                    let java_runner = self.settings_state.java.clone();

                                    let should_load_mods = profile.profile.loader().is_fabric();
                                    let profile_id = profile.profile.id;

                                    let run_game = Task::new(
                                        "Running the game",
                                        Caller::standard(async move {
                                            if should_load_mods {
                                                load_mods(profile_id).await.report_error();
                                            }

                                            instance.launch(user_data, &java_runner).await.report_error()
                                        }),
                                    );

                                    self.manager.push_task::<GameRunnerCollection>(run_game)
                                }
                            }
                            ProfileState::NotDownloaded { .. } => {
                                if ui
                                    .add_enabled(
                                        !self.profiles_state.currently_downloading_profiles.contains(&profile.profile.id),
                                        egui::Button::new("Download"),
                                    )
                                    .clicked()
                                {
                                    self.profiles_state.currently_downloading_profiles.insert(profile.profile.id);

                                    let game_version = profile.profile.version().to_owned();

                                    let assets_task = Task::new(
                                        format!("Assets ({})", profile.profile.version()),
                                        Caller::progressing(|progress| task_assets(game_version, PathBuf::from("./minecraft/assets"), progress)),
                                    );
                                    self.manager.push_task::<AssetsCollection>(assets_task);

                                    let profile = profile.clone();

                                    let game_task = Task::new(
                                        format!("Downloading version {}", profile.profile.version()),
                                        Caller::progressing(|progress| task_download_version(profile, progress)),
                                    );
                                    self.manager.push_task::<GameDownloadingCollection>(game_task);
                                }
                            }
                        });

                        row.col(|ui| {
                            if ui.button("Details").clicked() {
                                self.tabs_state.0.push(TabKind::ProfileInfo { profile: profile.clone() });
                            }
                        });

                        row.col(|ui| {
                            if let ProfileState::Downloaded(instance) = &profile.profile.state {
                                let popup_id = ui.make_persistent_id("delete_popup_id");
                                let button = ui
                                    .add_enabled(self.is_allowed_to_take_action, Button::new("Delete"))
                                    .on_hover_text("It will delete the profile and it's data");

                                if button.clicked() {
                                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                }

                                popup_below_widget(ui, popup_id, &button, PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                    ui.set_min_width(150.0);

                                    let delete_client_id = Id::new("delete_client");
                                    let delete_libraries_id = Id::new("delete_libraries");
                                    let delete_assets_id = Id::new("delete_assets");

                                    let mut make_checkbox = |text: &str, id, default: bool| {
                                        let mut state = ui.data_mut(|map| *map.get_temp_mut_or_insert_with(id, move || default));
                                        ui.checkbox(&mut state, text);
                                        ui.data_mut(|map| map.insert_temp(id, state));
                                    };

                                    make_checkbox("Delete profile's client", delete_client_id, true);
                                    make_checkbox("Delete profile's libraries", delete_libraries_id, true);
                                    make_checkbox("Delete profile's assets", delete_assets_id, false);

                                    ui.label("Are you sure you want to delete this profile and it's data?");
                                    ui.horizontal(|ui| {
                                        if ui.button("Yes").clicked() {
                                            is_deleting.push(index);

                                            let version = &instance.settings.version;

                                            let checkbox_data = |id| ui.data(|data| data.get_temp(id)).unwrap_or_default();

                                            let delete_client = checkbox_data(delete_client_id);
                                            let delete_libraries = checkbox_data(delete_libraries_id);
                                            let delete_assets = checkbox_data(delete_assets_id);

                                            let instance = instance.clone();
                                            let caller = Caller::standard(async move {
                                                instance.delete(delete_client, delete_libraries, delete_assets).await.report_error();
                                            });

                                            let task = Task::new(format!("Deleting the game's files ({})", version), caller);

                                            self.manager.push_task::<GameDeletionCollection>(task);

                                            ui.memory_mut(|mem| mem.close_popup());
                                        }
                                        if ui.button("No").clicked() {
                                            ui.memory_mut(|mem| mem.close_popup());
                                        }
                                    });
                                });
                            }
                        });
                    });
                }

                is_deleting.drain(..).for_each(|index| {
                    self.profiles_state.profiles.profiles.remove(index);
                    self.profiles_state.profiles.update_config().report_error();
                });
            });
    }
}
