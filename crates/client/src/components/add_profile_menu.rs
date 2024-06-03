use eframe::egui::{self, Color32, RichText};
use nomi_core::{
    configs::profile::{Loader, ProfileState, VersionProfile},
    repository::{
        fabric_meta::{get_fabric_versions, FabricVersions},
        launcher_manifest::{LauncherManifest, Version},
        manifest::VersionType,
    },
};

use crate::errors_pool::ErrorPoolExt;

use super::{download_progress::Task, profiles::ProfilesState, Component};

pub struct AddProfileMenu<'a> {
    pub launcher_manifest: &'a LauncherManifest,
    pub menu_state: &'a mut AddProfileMenuState,
    pub profiles_state: &'a mut ProfilesState,
}

pub struct AddProfileMenuState {
    selected_version_type: VersionType,

    profile_name_buf: String,
    selected_version_buf: Option<Version>,
    selected_loader_buf: Loader,

    task: Option<Task<FabricVersions>>,
    selected_fabric_version: Option<String>,
    fabric_versions: FabricVersions,
}

impl AddProfileMenuState {
    /// It will request available versions no matter which `Loader`
    /// is selected
    pub fn request_fabric_versions(&mut self) {
        if self.task.is_none() {
            self.task = Some(Task::new("Requesting available Fabric versions".to_owned()));
        }

        if let Some(task) = self.task.as_mut() {
            task.set_running(true)
        }

        let version = self.selected_version_buf.as_ref().unwrap().id.clone();

        let sender = self.task.as_ref().unwrap().result_channel().clone_tx();

        tokio::spawn(async move {
            if let Some(versions) = get_fabric_versions(version).await.report_error() {
                sender.send(versions).await.report_error();
            }
        });
    }
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
            selected_loader_buf: Loader::Vanilla,
            task: None,
            selected_fabric_version: None,
            fabric_versions: Vec::new(),
        }
    }
}

impl Component for AddProfileMenu<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        if let Some(Ok(versiosn)) = self
            .menu_state
            .task
            .as_mut()
            .map(|task| task.result_channel_mut())
            .map(|channel| channel.try_recv())
        {
            self.menu_state.fabric_versions = versiosn;
            if let Some(task) = self.menu_state.task.as_mut() {
                task.set_running(false)
            }
        }

        {
            ui.label("Profile name:");
            ui.text_edit_singleline(&mut self.menu_state.profile_name_buf);

            egui::ComboBox::from_label("Versions Filter")
                .selected_text(format!("{:?}", self.menu_state.selected_version_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.menu_state.selected_version_type,
                        VersionType::Release,
                        "Release",
                    );
                    ui.selectable_value(
                        &mut self.menu_state.selected_version_type,
                        VersionType::Snapshot,
                        "Snapshot",
                    );
                });

            let versions_iter = self.launcher_manifest.versions.iter();
            let versions = match self.menu_state.selected_version_type {
                VersionType::Release => versions_iter
                    .filter(|v| v.version_type == "release")
                    .collect::<Vec<_>>(),
                VersionType::Snapshot => versions_iter
                    .filter(|v| v.version_type == "snapshot")
                    .collect::<Vec<_>>(),
            };

            egui::ComboBox::from_label("Select version")
                .selected_text(
                    self.menu_state
                        .selected_version_buf
                        .as_ref()
                        .map_or("No version selcted", |v| &v.id),
                )
                .show_ui(ui, |ui| {
                    for version in versions {
                        let value = ui.selectable_value(
                            &mut self.menu_state.selected_version_buf.as_ref(),
                            Some(version),
                            &version.id,
                        );
                        if value.clicked() {
                            self.menu_state.selected_version_buf = Some(version.clone());
                            if matches!(self.menu_state.selected_loader_buf, Loader::Fabric) {
                                self.menu_state.request_fabric_versions()
                            }
                        }
                    }
                });

            ui.add_enabled_ui(self.menu_state.selected_version_buf.is_some(), |ui| {
                egui::ComboBox::from_label("Select the loader")
                    .selected_text(format!("{:?}", self.menu_state.selected_loader_buf))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.menu_state.selected_loader_buf,
                            Loader::Vanilla,
                            "Vanilla",
                        );
                        let fabric = ui.selectable_value(
                            &mut self.menu_state.selected_loader_buf,
                            Loader::Fabric,
                            "Fabric",
                        );

                        if fabric.clicked() {
                            println!("Test!");
                            self.menu_state.request_fabric_versions();
                        }
                    });
            });

            if matches!(self.menu_state.selected_loader_buf, Loader::Fabric) {
                ui.label(
                    RichText::new("Warn: Fabric version will not run if you have not installed Vanilla version previously")
                        .color(ui.visuals().warn_fg_color),
                );

                if !self.menu_state.fabric_versions.is_empty() {
                    egui::ComboBox::from_label("Select Fabric version")
                        .selected_text(
                            self.menu_state
                                .selected_fabric_version
                                .as_deref()
                                .unwrap_or("No version selected"),
                        )
                        .show_ui(ui, |ui| {
                            for version in &self.menu_state.fabric_versions {
                                let stability_text = match version.loader.stable {
                                    true => "stable",
                                    false => "unstable",
                                };

                                let stability_color = match version.loader.stable {
                                    true => Color32::GREEN,
                                    false => ui.visuals().warn_fg_color,
                                };
                                ui.horizontal(|ui| {
                                    ui.selectable_value(
                                        &mut self.menu_state.selected_fabric_version,
                                        Some(version.loader.version.clone()),
                                        RichText::new(&version.loader.version)
                                            .color(stability_color),
                                    );
                                    ui.label(RichText::new("❓").color(stability_color))
                                        .on_hover_text(stability_text);
                                });
                            }
                        });
                } else if self
                    .menu_state
                    .task
                    .as_ref()
                    .is_some_and(|task| task.is_running())
                {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Requesting available Fabric versions");
                    });
                } else {
                    ui.label(
                        RichText::new("Fabric is not available for this version")
                            .color(ui.visuals().error_fg_color),
                    );
                }
            }
        }

        let some_version_buf = || self.menu_state.selected_version_buf.is_some();
        let loader_is = |kind| self.menu_state.selected_loader_buf == kind;
        let some_fabric_version = || self.menu_state.selected_fabric_version.is_some();
        let fabric_versions_non_empty = || !self.menu_state.fabric_versions.is_empty();

        if self.menu_state.selected_version_buf.is_none() {
            ui.label(
                RichText::new("You must select the version").color(ui.visuals().error_fg_color),
            );
        }

        if self.menu_state.selected_fabric_version.is_none() && loader_is(Loader::Fabric) {
            ui.label(
                RichText::new("You must select the Fabric Version")
                    .color(ui.visuals().error_fg_color),
            );
        }

        if ui
            .add_enabled(
                some_version_buf()
                    && (loader_is(Loader::Vanilla)
                        || (loader_is(Loader::Fabric)
                            && some_fabric_version()
                            && fabric_versions_non_empty())),
                egui::Button::new("Create"),
            )
            .clicked()
        {
            self.profiles_state.add_profile(VersionProfile {
                id: self.profiles_state.create_id(),
                name: self.menu_state.profile_name_buf.clone(),
                state: ProfileState::NotDownloaded {
                    // PANICS: It will never panic because it's
                    // unreachable if `selected_version_buf` is `None`
                    version: self.menu_state.selected_version_buf.clone().unwrap().id,
                    loader: self.menu_state.selected_loader_buf.clone(),
                    version_type: self.menu_state.selected_version_type.clone(),
                },
            });
            self.profiles_state.update_config().report_error();
        }
    }
}
