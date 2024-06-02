use eframe::egui::{self, RichText};
use nomi_core::{
    configs::profile::{Loader, ProfileState, VersionProfile},
    repository::{
        launcher_manifest::{LauncherManifest, Version},
        manifest::VersionType,
    },
};

use crate::errors_pool::ErrorPoolExt;

use super::{profiles::ProfilesState, Component};

pub struct AddProfileMenu<'a> {
    pub launcher_manifest: &'a LauncherManifest,
    pub state: &'a mut AddProfileMenuState,
    pub profiles_state: &'a mut ProfilesState,
}

#[derive(Clone)]
pub struct AddProfileMenuState {
    selected_version_type: VersionType,

    profile_name_buf: String,
    selected_version_buf: Option<Version>,
    loader_buf: Loader,
}

impl Default for AddProfileMenuState {
    fn default() -> Self {
        Self::default_const()
    }
}

impl AddProfileMenuState {
    pub const fn default_const() -> Self {
        Self {
            selected_version_type: VersionType::Release,

            profile_name_buf: String::new(),
            selected_version_buf: None,
            loader_buf: Loader::Vanilla,
        }
    }
}

impl Component for AddProfileMenu<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        {
            ui.label("Profile name:");
            ui.text_edit_singleline(&mut self.state.profile_name_buf);

            egui::ComboBox::from_label("Versions Filter")
                .selected_text(format!("{:?}", self.state.selected_version_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.state.selected_version_type,
                        VersionType::Release,
                        "Release",
                    );
                    ui.selectable_value(
                        &mut self.state.selected_version_type,
                        VersionType::Snapshot,
                        "Snapshot",
                    );
                });

            let versions_iter = self.launcher_manifest.versions.iter();
            let versions = match self.state.selected_version_type {
                VersionType::Release => versions_iter
                    .filter(|v| v.version_type == "release")
                    .collect::<Vec<_>>(),
                VersionType::Snapshot => versions_iter
                    .filter(|v| v.version_type == "snapshot")
                    .collect::<Vec<_>>(),
            };

            egui::ComboBox::from_label("Select version")
                .selected_text(
                    self.state
                        .selected_version_buf
                        .as_ref()
                        .map_or("No version selcted", |v| &v.id),
                )
                .show_ui(ui, |ui| {
                    for version in versions {
                        let value = ui.selectable_value(
                            &mut self.state.selected_version_buf.as_ref(),
                            Some(version),
                            &version.id,
                        );
                        if value.clicked() {
                            self.state.selected_version_buf = Some(version.clone())
                        }
                    }
                });

            ui.horizontal(|ui| {
                ui.radio_value(&mut self.state.loader_buf, Loader::Vanilla, "Vanilla");
                ui.radio_value(&mut self.state.loader_buf, Loader::Fabric, "Fabric")
            });

            if matches!(self.state.loader_buf, Loader::Fabric) {
                ui.label(
                    RichText::new("You must install Vanilla before Fabric")
                        .color(ui.visuals().warn_fg_color),
                );
            }
        }

        if self.state.selected_version_buf.is_none() {
            ui.label(
                RichText::new("You must select the version").color(ui.visuals().warn_fg_color),
            );
        }
        if ui
            .add_enabled(
                self.state.selected_version_buf.is_some(),
                egui::Button::new("Create"),
            )
            .clicked()
        {
            self.profiles_state.add_profile(VersionProfile {
                id: self.profiles_state.create_id(),
                name: self.state.profile_name_buf.clone(),
                state: ProfileState::NotDownloaded {
                    // PANICS: It will never panic because it's
                    // unreachable if `selected_version_buf` is `None`
                    version: self.state.selected_version_buf.clone().unwrap().id,
                    loader: self.state.loader_buf.clone(),
                    version_type: self.state.selected_version_type.clone(),
                },
            });
            self.profiles_state.update_config().report_error();
        }
    }
}
