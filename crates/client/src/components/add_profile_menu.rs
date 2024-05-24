use eframe::egui::{self, Color32, RichText};
use nomi_core::{
    configs::profile::{Loader, ProfileState, VersionProfile},
    repository::launcher_manifest::{LauncherManifest, Version},
};

use crate::Storage;

use super::{profiles::ProfilesData, Component, StorageCreationExt};

pub struct AddProfileMenu<'a> {
    pub storage: &'a mut Storage,
    pub launcher_manifest: &'a LauncherManifest,
}

#[derive(Clone)]
struct AddProfileMenuData {
    selected_version_filter: VersionFilter,

    profile_name_buf: String,
    selected_version_buf: Option<Version>,
    loader_buf: Loader,
}

impl StorageCreationExt for AddProfileMenu<'_> {
    fn extend(storage: &mut crate::Storage) -> anyhow::Result<()> {
        storage.insert(AddProfileMenuData {
            selected_version_filter: VersionFilter::Release,

            profile_name_buf: String::new(),
            selected_version_buf: None,
            loader_buf: Loader::Vanilla,
        });
        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum VersionFilter {
    Release,
    Snapshot,
}

impl Component for AddProfileMenu<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        {
            let profile_data = self.storage.get_mut::<AddProfileMenuData>().unwrap();

            ui.label("Profile name:");
            ui.text_edit_singleline(&mut profile_data.profile_name_buf);

            egui::ComboBox::from_label("Versions Filter")
                .selected_text(format!("{:?}", profile_data.selected_version_filter))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut profile_data.selected_version_filter,
                        VersionFilter::Release,
                        "Release",
                    );
                    ui.selectable_value(
                        &mut profile_data.selected_version_filter,
                        VersionFilter::Snapshot,
                        "Snapshot",
                    );
                });

            let versions_iter = self.launcher_manifest.versions.iter();
            let versions = match profile_data.selected_version_filter {
                VersionFilter::Release => versions_iter
                    .filter(|v| v.version_type == "release")
                    .collect::<Vec<_>>(),
                VersionFilter::Snapshot => versions_iter
                    .filter(|v| v.version_type == "snapshot")
                    .collect::<Vec<_>>(),
            };

            egui::ComboBox::from_label("Select version")
                .selected_text(
                    profile_data
                        .selected_version_buf
                        .as_ref()
                        .map_or("No version selcted", |v| &v.id),
                )
                .show_ui(ui, |ui| {
                    for version in versions {
                        let value = ui.selectable_value(
                            &mut profile_data.selected_version_buf.as_ref(),
                            Some(version),
                            &version.id,
                        );
                        if value.clicked() {
                            profile_data.selected_version_buf = Some(version.clone())
                        }
                    }
                });

            ui.horizontal(|ui| {
                ui.radio_value(&mut profile_data.loader_buf, Loader::Vanilla, "Vanilla");
                ui.radio_value(&mut profile_data.loader_buf, Loader::Fabric, "Fabric")
            });
            ui.label(
                RichText::new("You must install vanilla before Fabric").color(Color32::YELLOW),
            );
        }

        let profile_data = self.storage.get_boxed::<AddProfileMenuData>().unwrap();
        if ui.button("Create").clicked() && profile_data.selected_version_buf.is_some() {
            let profiles = self.storage.get_mut::<ProfilesData>().unwrap();
            profiles.profiles.add_profile(VersionProfile {
                id: profiles.profiles.create_id(),
                name: profile_data.profile_name_buf,
                state: ProfileState::NotDownloaded {
                    version: profile_data.selected_version_buf.unwrap().id,
                    loader: profile_data.loader_buf,
                },
            });
        }
    }
}
