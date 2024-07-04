use std::{fmt::format, future::Future, sync::Arc};

use eframe::egui::{self, AboveOrBelow, Align2, Id, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    fs::write_toml_config_sync,
    instance::launch::arguments::UserData,
    repository::{launcher_manifest::LauncherManifest, username::Username},
    DOT_NOMI_PROFILES_CONFIG,
};
use serde::{Deserialize, Serialize};

use crate::{download::spawn_download, errors_pool::ErrorPoolExt, popup::popup, utils::spawn_tokio_future};

use super::{
    add_profile_menu::{AddProfileMenu, AddProfileMenuState},
    tasks_manager::{AssetsExtra, TasksManagerState, Task},
    settings::SettingsState,
    Component,
};

pub struct ProfilesPage<'a> {
    pub download_progress: &'a mut TasksManagerState,
    pub settings_state: &'a SettingsState,

    pub is_profile_window_open: &'a mut bool,

    pub profiles_state: &'a mut ProfilesState,
    pub menu_state: &'a mut AddProfileMenuState,

    pub launcher_manifest: &'static LauncherManifest,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ProfilesState {
    pub profiles: Vec<Arc<VersionProfile>>,
}

impl ProfilesState {
    pub fn add_profile(&mut self, profile: VersionProfile) {
        self.profiles.insert(self.create_id(), profile.into());
    }

    pub fn create_id(&self) -> usize {
        match &self.profiles.iter().max_by_key(|profile| profile.id) {
            Some(v) => v.id + 1,
            None => 0,
        }
    }

    pub fn update_config(&self) -> anyhow::Result<()> {
        write_toml_config_sync(&self, DOT_NOMI_PROFILES_CONFIG)
    }
}

impl Component for ProfilesPage<'_> {
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
                        // is_profile_window_open: self.is_profile_window_open,
                    }
                    .ui(ui);
                });
        }

        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

        TableBuilder::new(ui)
            .column(Column::auto().at_least(120.0).at_most(240.0))
            .columns(Column::auto(), 4)
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

                for (index, profile) in self.profiles_state.profiles.iter().enumerate() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Label::new(&profile.name).truncate());
                        });
                        row.col(|ui| {
                            ui.label(profile.version());
                        });
                        row.col(|ui| {
                            ui.label(profile.loader_name());
                        });
                        row.col(|ui| match &profile.state {
                            ProfileState::Downloaded(instance) => {
                                if ui
                                    .add_enabled(
                                        self.download_progress.is_allowed_to_take_action,
                                        egui::Button::new("Launch"),
                                    )
                                    .clicked()
                                {
                                    let instance = instance.clone();
                                    let (tx, _rx) = tokio::sync::mpsc::channel(100);

                                    let user_data = UserData {
                                        username: Username::new(
                                            self.settings_state.username.clone(),
                                        )
                                        .unwrap(),
                                        uuid: Some(self.settings_state.uuid.clone()),
                                        access_token: None,
                                    };

                                    let java_runner = self.settings_state.java.clone();

                                    spawn_tokio_future(tx, async move {
                                        instance
                                            .launch(user_data, &java_runner)
                                            .await
                                            .report_error()
                                    });
                                }
                            }
                            ProfileState::NotDownloaded { .. } => {
                                if ui
                                    .add_enabled(
                                        !self
                                            .download_progress
                                            .profile_tasks
                                            .contains_key(&profile.id),
                                        egui::Button::new("Download"),
                                    )
                                    .clicked()
                                {
                                    let version_task = Task::new(profile.version().to_owned());
                                    let id = profile.id;

                                    self.download_progress.assets_to_download.push_back(
                                        Task::new(format!("Assets ({})", profile.version()))
                                            .with_extra(AssetsExtra {
                                                version: profile.version().to_owned(),
                                                assets_dir: std::env::current_dir()
                                                    .report_error_with_context(
                                                        "Unable to get current directory",
                                                    )
                                                    .unwrap()
                                                    .join("minecraft")
                                                    .join("assets"),
                                            }),
                                    );

                                    let handle = spawn_download(
                                        profile.clone(),
                                        version_task.result_channel().clone_tx(),
                                        version_task.progress_channel().clone_tx(),
                                        version_task.total_channel().clone_tx(),
                                    );

                                    self.download_progress
                                        .profile_tasks
                                        .insert(id, version_task.with_handle(handle));
                                }
                            }
                        });

                        row.col(|ui| {
                            if let ProfileState::Downloaded(instance) = &profile.state {
                                let popup_id = ui.make_persistent_id("delete_popup_id");
                                let button = ui
                                    .button("Delete")
                                    .on_hover_text("It will delete the profile and it's data");

                                popup(ui, popup_id, &button, AboveOrBelow::Below, |ui, popup| {
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
                                            // let checkbox_data = |id| ui.data(|data| data.get_temp(id)).unwrap_or_default();

                                            // let task = Task::new(format!("Deleting the game's files ({})", &instance.settings.version));

                                            

                                            // self.download_progress.push_task(task);

                                            popup.close()
                                        }
                                        if ui.button("No").clicked() {
                                            popup.close()
                                        }
                                    });
                                });
                            }
                        });
                    });
                }

                is_deleting.drain(..).for_each(|index| {
                    self.profiles_state.profiles.remove(index);
                });
            });
    }
}

