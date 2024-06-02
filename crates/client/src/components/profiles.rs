use std::sync::Arc;

use eframe::egui::{self, Align2, Ui};
use egui_extras::{Column, TableBuilder};
use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    fs::write_toml_config_sync,
    repository::launcher_manifest::LauncherManifest,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{download::spawn_download, errors_pool::ErrorPoolExt, utils::spawn_tokio_future};

use super::{
    add_profile_menu::{AddProfileMenu, AddProfileMenuState},
    download_progress::{AssetsExtra, DownloadProgressState, Task},
    Component,
};

pub struct ProfilesPage<'a> {
    pub download_progress: &'a mut DownloadProgressState,

    pub is_profile_window_open: &'a mut bool,

    pub state: &'a mut ProfilesState,
    pub menu_state: &'a mut AddProfileMenuState,

    pub launcher_manifest: &'static LauncherManifest,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ProfilesState {
    pub profiles: Vec<Arc<VersionProfile>>,
}

impl ProfilesState {
    pub const fn default_const() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }

    pub fn add_profile(&mut self, profile: VersionProfile) {
        self.profiles.push(profile.into());
    }

    /// Create an id for the profile
    /// depends on the last id in the vector
    pub fn create_id(&self) -> u32 {
        match &self.profiles.iter().max_by_key(|x| x.id) {
            Some(v) => v.id + 1,
            None => 0,
        }
    }

    pub fn update_config(&self) -> anyhow::Result<()> {
        write_toml_config_sync(&self, "./.nomi/configs/Profiles.toml")
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
                        state: self.menu_state,
                        profiles_state: self.state,
                        launcher_manifest: self.launcher_manifest,
                        // is_profile_window_open: self.is_profile_window_open,
                    }
                    .ui(ui);
                });
        }

        ui.style_mut().wrap = Some(false);

        TableBuilder::new(ui)
            .column(Column::auto().at_most(120.0))
            .columns(Column::auto(), 2)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("Name");
                });
                header.col(|ui| {
                    ui.label("Version");
                });
            })
            .body(|mut body| {
                for profile in self.state.profiles.iter().cloned() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Label::new(&profile.name).truncate(true));
                        });
                        row.col(|ui| {
                            ui.label(profile.version());
                        });
                        row.col(|ui| match &profile.state {
                            ProfileState::Downloaded(instance) => {
                                if ui.button("Launch").clicked() {
                                    let instance = instance.clone();
                                    let (tx, _rx) = tokio::sync::mpsc::channel(100);
                                    spawn_tokio_future(tx, async move {
                                        instance
                                            .launch()
                                            .await
                                            .inspect_err(|e| error!("{}", e))
                                            .report_error()
                                    });
                                }
                            }
                            ProfileState::NotDownloaded { .. } => {
                                if ui.button("Download").clicked() {
                                    let version_task = Task::new(profile.version().to_owned());
                                    let id = profile.id;

                                    self.download_progress.assets_to_download.push(
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

                                    spawn_download(
                                        profile,
                                        version_task.result_channel().clone_tx(),
                                        version_task.progress_channel().clone_tx(),
                                        version_task.total_channel().clone_tx(),
                                    );

                                    self.download_progress.tasks.insert(id, version_task);
                                }
                            }
                        });
                    });
                }
            });
    }
}
