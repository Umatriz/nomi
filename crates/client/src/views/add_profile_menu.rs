use std::sync::Arc;

use eframe::egui::{self, Color32, RichText};
use egui_task_manager::{Caller, Task, TaskManager};
use nomi_core::{
    configs::profile::{Loader, ProfileState, VersionProfile},
    fs::write_toml_config_sync,
    game_paths::GamePaths,
    instance::{Instance, ProfilePayload},
    repository::{
        fabric_meta::{get_fabric_versions, FabricVersions},
        launcher_manifest::{LauncherManifest, Version},
        manifest::VersionType,
    },
};
use parking_lot::RwLock;

use crate::{collections::FabricDataCollection, errors_pool::ErrorPoolExt, ui_ext::UiExt, views::ModdedProfile};

use super::{profiles::InstancesState, View};

pub struct AddProfileMenu<'a> {
    pub manager: &'a mut TaskManager,
    pub launcher_manifest: &'a LauncherManifest,
    pub menu_state: &'a mut AddProfileMenuState,
    pub profiles_state: &'a mut InstancesState,
}

pub struct AddProfileMenuState {
    instance_name: String,

    parent_instance: Option<Arc<RwLock<Instance>>>,

    selected_version_type: VersionType,

    profile_name_buf: String,
    selected_version_buf: Option<Version>,
    selected_loader_buf: Loader,

    pub fabric_versions: FabricVersions,
}

impl AddProfileMenuState {
    /// It will request available versions no matter which `Loader`
    /// is selected
    pub fn request_fabric_versions(&self, manager: &mut TaskManager) {
        let version = self.selected_version_buf.as_ref().unwrap().id.clone();

        let task = Task::new(
            "Requesting available Fabric versions",
            Caller::standard(async move { get_fabric_versions(version).await.report_error() }),
        );
        manager.push_task::<FabricDataCollection>(task);
    }
}

impl Default for AddProfileMenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl AddProfileMenuState {
    pub fn new() -> Self {
        Self {
            instance_name: String::new(),
            parent_instance: None,

            selected_version_type: VersionType::Release,

            profile_name_buf: String::new(),
            selected_version_buf: None,
            selected_loader_buf: Loader::Vanilla,
            fabric_versions: Vec::new(),
        }
    }
}

impl View for AddProfileMenu<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        fn fabric_version_is(selected_loader: &Loader, loader: Loader, func: impl Fn(Option<&String>) -> bool) -> bool {
            matches!(loader, Loader::Fabric { .. })
                && match selected_loader {
                    Loader::Fabric { version } => func(version.as_ref()),
                    Loader::Vanilla => unreachable!(),
                    Loader::Forge => unreachable!(),
                }
        }

        egui::ComboBox::from_label("Select instance to create profile for")
            .selected_text(
                self.menu_state
                    .parent_instance
                    .as_ref()
                    .map_or(String::from("No instance selected"), |i| i.read().name().to_owned()),
            )
            .show_ui(ui, |ui| {
                for instance in &self.profiles_state.instances.instances {
                    if ui
                        .selectable_label(
                            self.menu_state
                                .parent_instance
                                .as_ref()
                                .is_some_and(|i| i.read().id() == instance.read().id()),
                            instance.read().name(),
                        )
                        .clicked()
                    {
                        self.menu_state.parent_instance = Some(instance.clone());
                    }
                }
            });

        {
            ui.label("Profile name:");
            ui.text_edit_singleline(&mut self.menu_state.profile_name_buf);

            egui::ComboBox::from_label("Versions Filter")
                .selected_text(format!("{:?}", self.menu_state.selected_version_type))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.menu_state.selected_version_type, VersionType::Release, "Release");
                    ui.selectable_value(&mut self.menu_state.selected_version_type, VersionType::Snapshot, "Snapshot");
                });

            let versions_iter = self.launcher_manifest.versions.iter();
            let versions = match &self.menu_state.selected_version_type {
                VersionType::Release => versions_iter.filter(|v| v.version_type == "release").collect::<Vec<_>>(),
                VersionType::Snapshot => versions_iter.filter(|v| v.version_type == "snapshot").collect::<Vec<_>>(),
            };

            egui::ComboBox::from_label("Select version")
                .selected_text(self.menu_state.selected_version_buf.as_ref().map_or("No version selected", |v| &v.id))
                .show_ui(ui, |ui| {
                    for version in versions {
                        let value = ui.selectable_value(&mut self.menu_state.selected_version_buf.as_ref(), Some(version), &version.id);
                        if value.clicked() {
                            self.menu_state.selected_version_buf = Some(version.clone());
                            if matches!(self.menu_state.selected_loader_buf, Loader::Fabric { .. }) {
                                self.menu_state.request_fabric_versions(self.manager)
                            }
                        }
                    }
                });

            ui.add_enabled_ui(self.menu_state.selected_version_buf.is_some(), |ui| {
                egui::ComboBox::from_label("Select the loader")
                    .selected_text(format!("{}", self.menu_state.selected_loader_buf))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.menu_state.selected_loader_buf, Loader::Vanilla, "Vanilla");
                        ui.selectable_value(&mut self.menu_state.selected_loader_buf, Loader::Forge, "Forge");
                        let fabric = ui.selectable_value(&mut self.menu_state.selected_loader_buf, Loader::Fabric { version: None }, "Fabric");

                        if fabric.clicked() {
                            println!("Test!");
                            self.menu_state.request_fabric_versions(self.manager);
                        }
                    });
            });

            if matches!(self.menu_state.selected_loader_buf, Loader::Fabric { .. }) {
                ui.label(
                    RichText::new("Warn: Fabric version will not run if you have not installed Vanilla version previously")
                        .color(ui.visuals().warn_fg_color),
                );

                if !self.menu_state.fabric_versions.is_empty() {
                    if let Loader::Fabric { version } = &mut self.menu_state.selected_loader_buf {
                        egui::ComboBox::from_label("Select Fabric version")
                            .selected_text(version.as_deref().unwrap_or("No version selected"))
                            .show_ui(ui, |ui| {
                                for fabric_version in &self.menu_state.fabric_versions {
                                    let stability_text = match fabric_version.loader.stable {
                                        true => "stable",
                                        false => "unstable",
                                    };

                                    let stability_color = match fabric_version.loader.stable {
                                        true => Color32::GREEN,
                                        false => ui.visuals().warn_fg_color,
                                    };
                                    ui.horizontal(|ui| {
                                        ui.selectable_value(
                                            version,
                                            Some(fabric_version.loader.version.clone()),
                                            RichText::new(&fabric_version.loader.version).color(stability_color),
                                        );
                                        ui.label(RichText::new("❓").color(stability_color)).on_hover_text(stability_text);
                                    });
                                }
                            });
                    }
                } else if !self.manager.get_collection::<FabricDataCollection>().tasks().is_empty() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Requesting available Fabric versions");
                    });
                } else {
                    ui.label(RichText::new("Fabric is not available for this version").color(ui.visuals().error_fg_color));
                }
            }
        }

        let some_version_buf = || self.menu_state.selected_version_buf.is_some();

        let fabric_version_is_some = || {
            fabric_version_is(
                &self.menu_state.selected_loader_buf,
                self.menu_state.selected_loader_buf.clone(),
                |opt: Option<&String>| opt.is_some(),
            )
        };
        let fabric_version_is_none = || {
            fabric_version_is(
                &self.menu_state.selected_loader_buf,
                self.menu_state.selected_loader_buf.clone(),
                |opt: Option<&String>| opt.is_none(),
            )
        };

        let fabric_versions_non_empty = || !self.menu_state.fabric_versions.is_empty();

        if self.menu_state.profile_name_buf.trim().is_empty() {
            ui.error_label("You must enter the profile name");
        }

        if self.menu_state.selected_version_buf.is_none() {
            ui.error_label("You must select the version");
        }

        if fabric_version_is_none() {
            ui.error_label("You must select the Fabric Version");
        }

        if self.menu_state.parent_instance.is_none() {
            ui.error_label("You must select the instance to create profile for");
        }

        if ui
            .add_enabled(
                self.menu_state.parent_instance.is_some()
                    && some_version_buf()
                    && (matches!(self.menu_state.selected_loader_buf, Loader::Vanilla)
                        || matches!(self.menu_state.selected_loader_buf, Loader::Forge)
                        || (fabric_version_is_some() && fabric_versions_non_empty())),
                egui::Button::new("Create"),
            )
            .clicked()
        {
            if let Some(instance) = &self.menu_state.parent_instance {
                let payload = {
                    let instance = instance.read();
                    let version = self.menu_state.selected_version_buf.clone().unwrap().id;
                    let profile = VersionProfile {
                        id: instance.next_id(),
                        name: self.menu_state.profile_name_buf.trim_end().to_owned(),
                        state: ProfileState::NotDownloaded {
                            // PANICS: It will never panic because it's
                            // unreachable for `selected_version_buf` to be `None`
                            version: version.clone(),
                            loader: self.menu_state.selected_loader_buf.clone(),
                            version_type: self.menu_state.selected_version_type.clone(),
                        },
                    };

                    let path = GamePaths::from_instance_path(instance.path(), profile.id.profile()).profile_config();

                    let payload = ProfilePayload::from_version_profile(&profile, &path);
                    let profile = ModdedProfile::new(profile);

                    write_toml_config_sync(&profile, path).report_error();

                    payload
                };

                {
                    let id = {
                        let mut instance = instance.write();
                        instance.add_profile(payload);
                        instance.id()
                    };
                    self.profiles_state.instances.update_instance_config(id).report_error();
                }
            }
        }
    }
}
