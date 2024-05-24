use eframe::egui::{self, Ui};
use egui_extras::{Column, TableBuilder};
use nomi_core::{
    configs::profile::ProfileState, fs::read_toml_config_sync,
    repository::launcher_manifest::LauncherManifest,
};

use crate::Storage;

use super::{add_profile_menu::AddProfileMenu, Component, StorageCreationExt};

pub struct ProfilesPage<'a> {
    pub storage: &'a mut Storage,
    pub launcher_manifest: &'static LauncherManifest,
}

pub(super) struct ProfilesData {
    pub(super) profiles: nomi_core::configs::profile::VersionProfilesConfig,
}

impl StorageCreationExt for ProfilesPage<'_> {
    fn extend(storage: &mut Storage) -> anyhow::Result<()> {
        let profiles = read_toml_config_sync::<nomi_core::configs::profile::VersionProfilesConfig>(
            "./.nomi/configs/Profiles.toml",
        )
        .unwrap_or_default();

        storage.insert(ProfilesData { profiles });

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
                for profile in &profiles.profiles.profiles {
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
                                    println!("Downloading!")
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
