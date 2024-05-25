use std::sync::Arc;

use eframe::egui::{self, Ui};
use egui_extras::{Column, TableBuilder};
use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    downloads::traits::DownloadResult,
    fs::{read_toml_config_sync, write_toml_config_sync},
    repository::launcher_manifest::LauncherManifest,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use crate::{download::spawn_download, Storage};

use super::{add_profile_menu::AddProfileMenu, Component, StorageCreationExt};

pub struct ProfilesPage<'a> {
    pub download_result_tx: Sender<VersionProfile>,
    pub download_progress_tx: Sender<DownloadResult>,
    pub download_total_tx: Sender<u32>,

    pub storage: &'a mut Storage,
    pub launcher_manifest: &'static LauncherManifest,
}

#[derive(Serialize, Deserialize, Default)]
pub(super) struct ProfilesData {
    pub(super) profiles: Vec<Arc<VersionProfile>>,
}

impl ProfilesData {
    pub fn add_profile(&mut self, profile: VersionProfile) {
        self.profiles.push(profile.into());
    }

    /// Create an id for the profile
    /// depends on the last id in the vector
    pub fn create_id(&self) -> i32 {
        match &self.profiles.iter().max_by_key(|x| x.id) {
            Some(v) => v.id + 1,
            None => 0,
        }
    }

    pub fn update_config(&self) -> anyhow::Result<()> {
        write_toml_config_sync(&self, "./.nomi/configs/Profiles.toml")
    }
}

impl StorageCreationExt for ProfilesPage<'_> {
    fn extend(storage: &mut Storage) -> anyhow::Result<()> {
        let profiles = read_toml_config_sync::<ProfilesData>("./.nomi/configs/Profiles.toml")
            .unwrap_or_default();

        storage.insert(profiles);

        Ok(())
    }
}

impl Component for ProfilesPage<'_> {
    fn ui(self, ui: &mut Ui) {
        let profiles = self
            .storage
            .get::<ProfilesData>()
            .expect("`ProfilesData::extend` must be called in the `Context::new`");

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
                for profile in profiles.profiles.iter().cloned() {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::Label::new(&profile.name).truncate(true));
                        });
                        row.col(|ui| {
                            ui.label(profile.version());
                        });
                        row.col(|ui| match &profile.state {
                            ProfileState::Downloaded(_) => {
                                if ui.button("Launch").clicked() {
                                    println!("Clicked!")
                                }
                            }
                            ProfileState::NotDownloaded { .. } => {
                                if ui.button("Download").clicked() {
                                    // TODO: reset `download_progress.current` each download
                                    spawn_download(
                                        profile,
                                        self.download_result_tx.clone(),
                                        self.download_progress_tx.clone(),
                                        self.download_total_tx.clone(),
                                    );
                                }
                            }
                        });
                    });
                }
            });

        AddProfileMenu {
            storage: self.storage,
            launcher_manifest: self.launcher_manifest,
        }
        .ui(ui);
    }
}
