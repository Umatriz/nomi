use std::{collections::HashSet, path::PathBuf, sync::Arc};

use anyhow::bail;
use eframe::egui::{self, Align2, RichText, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use egui_task_manager::{Caller, Task, TaskManager};
use itertools::Itertools;
use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    fs::write_toml_config_sync,
    game_paths::GamePaths,
    instance::{launch::arguments::UserData, load_instances, Instance, InstanceProfileId, ProfilePayload},
    repository::{launcher_manifest::LauncherManifest, username::Username},
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    cache::GLOBAL_CACHE,
    collections::{AssetsCollection, GameDownloadingCollection, GameRunnerCollection},
    download::{task_assets, task_download_version},
    errors_pool::ErrorPoolExt,
    TabKind,
};

use super::{add_profile_menu::AddProfileMenuState, load_mods, settings::SettingsState, LogsState, ModsConfig, ProfileInfoState, TabsState, View};

pub struct Instances<'a> {
    pub is_allowed_to_take_action: bool,
    pub manager: &'a mut TaskManager,
    pub settings_state: &'a SettingsState,
    pub profile_info_state: &'a mut ProfileInfoState,

    pub logs_state: &'a LogsState,
    pub tabs_state: &'a mut TabsState,
    pub profiles_state: &'a mut InstancesState,
    pub menu_state: &'a mut AddProfileMenuState,

    pub launcher_manifest: &'static LauncherManifest,
}

pub struct InstancesState {
    pub currently_downloading_profiles: HashSet<InstanceProfileId>,
    pub instances: InstancesConfig,
}

impl Default for InstancesState {
    fn default() -> Self {
        Self::new()
    }
}

impl InstancesState {
    pub fn new() -> Self {
        Self {
            currently_downloading_profiles: HashSet::new(),
            instances: InstancesConfig::load(),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct InstancesConfig {
    pub instances: Vec<Arc<RwLock<Instance>>>,
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

impl InstancesConfig {
    pub fn find_profile(&self, id: InstanceProfileId) -> Option<Arc<RwLock<ModdedProfile>>> {
        self.get_profile_path(id).and_then(|path| GLOBAL_CACHE.write().request_profile(id, path))
    }

    pub fn get_profile_path(&self, id: InstanceProfileId) -> Option<PathBuf> {
        self.find_instance(id.instance())
            .and_then(|i| i.read().profiles().iter().find(|p| p.id == id).map(|p| p.path.clone()))
    }

    pub fn find_instance(&self, id: usize) -> Option<Arc<RwLock<Instance>>> {
        self.instances.iter().find(|p| p.read().id() == id).cloned()
    }

    pub fn load() -> Self {
        Self {
            instances: load_instances()
                .unwrap_or_default()
                .into_iter()
                .map(RwLock::new)
                .map(Arc::new)
                .collect_vec(),
        }
    }

    pub async fn load_async() -> anyhow::Result<Self> {
        tokio::task::spawn_blocking(Self::load).await.map_err(Into::into)
    }

    pub fn add_instance(&mut self, instance: Instance) {
        self.instances.push(RwLock::new(instance).into())
    }

    pub fn next_id(&self) -> usize {
        match &self.instances.iter().map(|instance| instance.read().id()).max() {
            Some(id) => id + 1,
            None => 0,
        }
    }

    pub fn update_profile_config(&self, id: InstanceProfileId) -> anyhow::Result<()> {
        let Some((path, profile)) = self
            .get_profile_path(id)
            .and_then(|path| self.find_profile(id).map(|profile| (path, profile)))
        else {
            error!(?id, "Cannot find the profile");
            bail!("Cannot find ")
        };

        let profile = profile.read();
        write_toml_config_sync(&*profile, path)
    }

    pub fn update_all_instance_configs(&self) -> anyhow::Result<()> {
        for instance in self.instances.iter() {
            let instance = instance.read();
            instance.write_blocking().report_error();
        }

        Ok(())
    }

    pub fn update_instance_config(&self, id: usize) -> anyhow::Result<()> {
        let Some(instance) = self.find_instance(id) else {
            bail!("No such instance")
        };

        let instance = instance.read();

        instance.write_blocking()
    }
}

impl Instances<'_> {
    // TODO: It requires profile to be loaded
    fn profile_action_ui(&mut self, ui: &mut Ui, profile_payload: &ProfilePayload) {
        let button = if profile_payload.is_downloaded {
            ui.add_enabled(self.is_allowed_to_take_action, egui::Button::new("Launch"))
        } else {
            ui.add_enabled(
                !self.profiles_state.currently_downloading_profiles.contains(&profile_payload.id),
                egui::Button::new("Download"),
            )
        };

        if button.clicked() {
            self.profile_action(ui, profile_payload)
        }
    }

    fn profile_action(&mut self, ui: &mut Ui, profile_payload: &ProfilePayload) {
        let Some(profile_lock) = self.profiles_state.instances.find_profile(profile_payload.id) else {
            error!(id = ?profile_payload.id, "Cannot find the profile");
            return;
        };
        let profile = profile_lock.read();
        match &profile.profile.state {
            ProfileState::Downloaded(instance) => {
                let user_data = UserData {
                    username: Username::new(self.settings_state.username.clone()).unwrap(),
                    uuid: Some(self.settings_state.uuid.clone()),
                    access_token: None,
                };

                let instance = instance.clone();
                let java_runner = self.settings_state.java.clone();

                let should_load_mods = profile.profile.loader().support_mods();
                let profile_id = profile.profile.id;

                let game_logs = self.logs_state.game_logs.clone();
                game_logs.clear();
                let run_game = Task::new(
                    "Running the game",
                    Caller::standard(async move {
                        if should_load_mods {
                            load_mods(profile_id).await.report_error();
                        }

                        instance
                            .launch(GamePaths::from_id(profile_id), user_data, &java_runner, &*game_logs)
                            .await
                            .report_error()
                    }),
                );

                self.manager.push_task::<GameRunnerCollection>(run_game)
            }
            ProfileState::NotDownloaded { .. } => {
                self.profiles_state.currently_downloading_profiles.insert(profile.profile.id);

                let game_version = profile.profile.version().to_owned();

                let game_paths = GamePaths::from_id(profile.profile.id);
                let ctx = ui.ctx().clone();
                let assets_task = Task::new(
                    format!("Assets ({})", profile.profile.version()),
                    Caller::progressing(|progress| task_assets(progress, ctx, game_version, game_paths.assets)),
                );
                self.manager.push_task::<AssetsCollection>(assets_task);

                let profile_clone = profile_lock.clone();

                let id = profile.profile.id;
                let ctx = ui.ctx().clone();
                let java_runner = self.settings_state.java.clone();
                let game_task = Task::new(
                    format!("Downloading version {}", profile.profile.version()),
                    Caller::progressing(move |progress| async move {
                        task_download_version(progress, ctx, profile_clone, java_runner).await.map(|()| id)
                    }),
                );
                self.manager.push_task::<GameDownloadingCollection>(game_task);
            }
        }
    }

    fn show_profiles_for_instance(&mut self, ui: &mut Ui, profiles: &[ProfilePayload], is_allowed_to_take_action: bool) {
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
                // let mut is_deleting = vec![];

                for (_index, profile) in profiles.iter().enumerate() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Label::new(&profile.name).truncate());
                        });
                        row.col(|ui| {
                            ui.label(&profile.version);
                        });
                        row.col(|ui| {
                            ui.label(profile.loader.name());
                        });
                        row.col(|ui| self.profile_action_ui(ui, profile));

                        row.col(|ui| {
                            if ui.button("Details").clicked() {
                                if let Some(profile_lock) = self.profiles_state.instances.find_profile(profile.id) {
                                    self.profile_info_state.set_profile_to_edit(&profile_lock.read());

                                    let kind = TabKind::ProfileInfo {
                                        profile: profile_lock.clone(),
                                    };
                                    self.tabs_state.0.insert(kind.id(), kind);
                                };
                            }
                        });

                        row.col(|ui| {
                            ui.button("TODO: Delete");
                        });
                    });
                }

                // is_deleting.drain(..).for_each(|index| {
                //     self.profiles_state.instances.instances.remove(index);
                //     self.profiles_state.instances.update_config_sync().report_error();
                // });
            });
    }
}

impl View for Instances<'_> {
    fn ui(mut self, ui: &mut Ui) {
        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

        let iter = self.profiles_state.instances.instances.iter().cloned().collect_vec().into_iter();
        for instance in iter {
            let instance = instance.read();
            ui.group(|ui| {
                ui.label(RichText::new(instance.name()).strong());
                egui::CollapsingHeader::new("Profiles")
                    .id_source(egui::Id::new(instance.id()).with("__profiles_list"))
                    .show(ui, |ui| {
                        self.show_profiles_for_instance(ui, instance.profiles(), self.is_allowed_to_take_action)
                    });
            });
        }
    }
}

fn delete_profile_ui() {
    // if let ProfileState::Downloaded(instance) = &profile.profile.state {
    //     let popup_id = ui.make_persistent_id("delete_popup_id");
    //     let button = ui
    //         .add_enabled(is_allowed_to_take_action, Button::new("Delete"))
    //         .on_hover_text("It will delete the profile and it's data");

    //     if button.clicked() {
    //         ui.memory_mut(|mem| mem.toggle_popup(popup_id));
    //     }

    //     popup_below_widget(ui, popup_id, &button, PopupCloseBehavior::CloseOnClickOutside, |ui| {
    //         ui.set_min_width(150.0);

    //         let delete_client_id = Id::new("delete_client");
    //         let delete_libraries_id = Id::new("delete_libraries");
    //         let delete_assets_id = Id::new("delete_assets");
    //         let delete_mods_id = Id::new("delete_mods");

    //         let mut make_checkbox = |text: &str, id, default: bool| {
    //             let mut state = ui.data_mut(|map| *map.get_temp_mut_or_insert_with(id, move || default));
    //             ui.checkbox(&mut state, text);
    //             ui.data_mut(|map| map.insert_temp(id, state));
    //         };

    //         make_checkbox("Delete profile's client", delete_client_id, true);
    //         make_checkbox("Delete profile's libraries", delete_libraries_id, false);
    //         if profile.profile.loader().is_fabric() {
    //             make_checkbox("Delete profile's mods", delete_mods_id, true);
    //         }
    //         make_checkbox("Delete profile's assets", delete_assets_id, false);

    //         ui.label("Are you sure you want to delete this profile and it's data?");
    //         ui.horizontal(|ui| {
    //             ui.warn_icon_with_hover_text("Deleting profile's assets and libraries might break other profiles.");
    //             if ui.button("Yes").clicked() {
    //                 is_deleting.push(index);

    //                 let version = &instance.settings.version;

    //                 let checkbox_data = |id| ui.data(|data| data.get_temp(id)).unwrap_or_default();

    //                 let delete_client = checkbox_data(delete_client_id);
    //                 let delete_libraries = checkbox_data(delete_libraries_id);
    //                 let delete_assets = checkbox_data(delete_assets_id);
    //                 let delete_mods = checkbox_data(delete_mods_id);

    //                 let profile_id = profile.profile.id;

    //                 let instance = instance.clone();
    //                 let caller = Caller::standard(async move {
    //                     let path = Path::new(DOT_NOMI_MODS_STASH_DIR).join(format!("{}", profile_id));
    //                     if delete_mods && path.exists() {
    //                         tokio::fs::remove_dir_all(path).await.report_error();
    //                     }
    //                     instance.delete(delete_client, delete_libraries, delete_assets).await.report_error();
    //                 });

    //                 let task = Task::new(format!("Deleting the game's files ({})", version), caller);

    //                 self.manager.push_task::<GameDeletionCollection>(task);

    //                 self.tabs_state.remove_profile_related_tabs(&profile);

    //                 ui.memory_mut(|mem| mem.close_popup());
    //             }
    //             if ui.button("No").clicked() {
    //                 ui.memory_mut(|mem| mem.close_popup());
    //             }
    //         });
    //     });
    // }
}
