use eframe::egui::Ui;
use nomi_core::fs::read_toml_config_sync;

use crate::Storage;

use super::{Component, StorageCreationExt};

pub struct ProfilesPage;

struct ProfilesData {
    profiles: nomi_core::configs::profile::VersionProfilesConfig,
}

impl StorageCreationExt for ProfilesPage {
    fn extend(storage: &mut Storage) -> anyhow::Result<()> {
        let profiles = read_toml_config_sync::<nomi_core::configs::profile::VersionProfilesConfig>(
            "./.nomi/configs/Profiles.toml",
        )?;

        storage.insert(ProfilesData { profiles });

        Ok(())
    }
}

impl Component for ProfilesPage {
    fn ui(self, ui: &mut Ui, storage: &mut Storage) {
        let profiles = storage
            .get::<ProfilesData>()
            .expect("`ProfilesData::extend` must be called in the `Context::new`");

        ui.vertical(|ui| {
            for profile in profiles.profiles.profiles.iter() {
                ui.horizontal(|ui| {
                    ui.label(&profile.name);
                    ui.label(&profile.instance.settings.version);
                });
            }
        });
    }
}
