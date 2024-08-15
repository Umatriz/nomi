use std::{collections::HashSet, sync::Arc};

use eframe::egui::{self, Color32, Id, RichText, TextEdit};
use egui_task_manager::{Caller, Task, TaskManager};
use nomi_core::configs::profile::ProfileState;
use nomi_modding::modrinth::project::ProjectId;
use parking_lot::RwLock;

use crate::{
    collections::DownloadAddedModsCollection, errors_pool::ErrorPoolExt, open_directory::open_directory_native, toasts, ui_ext::UiExt,
    views::InstancesConfig, TabKind,
};

use super::{download_added_mod, mods_stash_path_for_profile, Mod, ModdedProfile, TabsState, View};

pub struct ProfileInfo<'a> {
    pub profiles: &'a InstancesConfig,
    pub task_manager: &'a mut TaskManager,
    pub profile: Arc<RwLock<ModdedProfile>>,
    pub tabs_state: &'a mut TabsState,
    pub profile_info_state: &'a mut ProfileInfoState,
}

#[derive(Default)]
pub struct ProfileInfoState {
    pub currently_downloading_mods: HashSet<ProjectId>,

    pub profile_name: String,
    pub profile_jvm_args: Vec<String>,
    pub jvm_arg_to_add: String,

    pub is_import_window_open: bool,
    pub mods_to_import_string: String,
    pub mods_to_import: Vec<Mod>,
    pub conflicts: Vec<ImportConflict>,

    pub is_export_window_open: bool,
    pub included_mods: Vec<bool>,
}

impl ProfileInfoState {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn proceed_mods_import(&mut self, profile: &ModdedProfile) {
        for incoming in &self.mods_to_import {
            let Some(existing) = profile.mods.mods.iter().find(|m| m.project_id == incoming.project_id) else {
                continue;
            };

            self.conflicts.push(ImportConflict {
                name: existing.name.clone(),
                existing: existing.clone(),
                incoming: incoming.clone(),
                resolved: None,
            })
        }
    }

    pub fn set_profile_to_edit(&mut self, profile: &ModdedProfile) {
        self.profile_name.clone_from(&profile.profile.name);

        if let ProfileState::Downloaded(instance) = &profile.profile.state {
            self.profile_jvm_args = instance.jvm_arguments().into();
        }
    }
}

pub struct ImportConflict {
    pub name: String,
    pub existing: Mod,
    pub incoming: Mod,
    pub resolved: Option<ImportConflictSolution>,
}

impl ImportConflict {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        fn show_variant(
            ui: &mut egui::Ui,
            is_selected: bool,
            conflict_resolved: &mut Option<ImportConflictSolution>,
            variant: &Mod,
            solution: ImportConflictSolution,
            grid_id: impl std::hash::Hash,
        ) {
            ui.vertical(|ui| {
                egui::Grid::new(Id::new(grid_id).with(ui.id())).show(ui, |ui| {
                    ui.label("Name:");
                    ui.label(&variant.name);
                    ui.end_row();

                    ui.label("Version number:");
                    ui.label(variant.version_number.as_deref().unwrap_or("None"));
                    ui.end_row();

                    ui.label("Version name:");
                    ui.label(variant.version_name.as_deref().unwrap_or("None"));
                    ui.end_row();

                    ui.label("Files:");
                    ui.vertical(|ui| {
                        for file in &variant.files {
                            ui.label(&file.filename);
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.button_with_confirm_popup(Id::new(&variant.version_id), "Accept", |ui| {
                        ui.label("Are you sure you wanna accept this variant to solve the conflict?");
                        ui.horizontal(|ui| {
                            let yes_button = ui.button("Yes");
                            let no_button = ui.button("No");

                            if yes_button.clicked() {
                                *conflict_resolved = Some(solution)
                            };

                            if yes_button.clicked() || no_button.clicked() {
                                ui.memory_mut(|mem| mem.close_popup());
                            }
                        });
                    });

                    if is_selected {
                        ui.colored_label(Color32::GREEN, "Selected");
                    }
                });
            });
        }

        ui.horizontal(|ui| {
            show_variant(
                ui,
                self.resolved.as_ref().is_some_and(|s| matches!(s, ImportConflictSolution::Existing)),
                &mut self.resolved,
                &self.existing,
                ImportConflictSolution::Existing,
                "conflict_existing_id",
            );
            ui.separator();
            show_variant(
                ui,
                self.resolved.as_ref().is_some_and(|s| matches!(s, ImportConflictSolution::Incoming)),
                &mut self.resolved,
                &self.incoming,
                ImportConflictSolution::Incoming,
                "conflict_incoming_id",
            );
        });
    }
}

pub enum ImportConflictSolution {
    Existing,
    Incoming,
}

fn proceed_conflicting_mods<'a>(conflicts: impl Iterator<Item = &'a ImportConflict>) -> impl Iterator<Item = &'a Mod> {
    conflicts.filter_map(|c| {
        c.resolved.as_ref().map(|resolved| match resolved {
            ImportConflictSolution::Existing => &c.existing,
            ImportConflictSolution::Incoming => &c.incoming,
        })
    })
}

fn mod_info_ui(ui: &mut egui::Ui, modification: &Mod) {
    ui.label(&modification.name);
    ui.label(modification.version_name.as_deref().unwrap_or("None"));
    ui.label(modification.version_number.as_deref().unwrap_or("None"));
}

impl View for ProfileInfo<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        egui::Window::new("Import mods")
            .open(&mut self.profile_info_state.is_import_window_open)
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label("Paste the import code");
                    let response = ui.text_edit_singleline(&mut self.profile_info_state.mods_to_import_string);
                    let button = ui.button("Check for conflicts");

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || button.clicked() {
                        if let Some(mods) = serde_json::from_str::<Vec<Mod>>(&self.profile_info_state.mods_to_import_string).report_error() {
                            for mut incoming in mods {
                                let profile = self.profile.read();
                                let Some(existing) = profile
                                    .mods
                                    .mods
                                    .iter()
                                    .find(|m| m.project_id == incoming.project_id && m.version_id != incoming.version_id)
                                else {
                                    if let Some(modification) = profile
                                        .mods
                                        .mods
                                        .iter()
                                        .find(|m| m.project_id == incoming.project_id && m.version_id == incoming.version_id)
                                    {
                                        incoming.is_downloaded = modification.is_downloaded;
                                    };
                                    self.profile_info_state.mods_to_import.push(incoming);
                                    continue;
                                };

                                self.profile_info_state.conflicts.push(ImportConflict {
                                    name: existing.name.clone(),
                                    existing: existing.clone(),
                                    incoming: incoming.clone(),
                                    resolved: None,
                                })
                            }
                        }
                    }

                    if !self.profile_info_state.conflicts.is_empty() {
                        ui.label(RichText::new("Conflicts").heading().strong());
                        ui.separator();

                        for conflict in &mut self.profile_info_state.conflicts {
                            ui.horizontal(|ui| {
                                let (color, text) = if conflict.resolved.is_some() {
                                    (Color32::GREEN, "✅")
                                } else {
                                    (Color32::RED, "❌")
                                };
                                ui.colored_label(color, text);

                                egui::CollapsingHeader::new(&conflict.name).show(ui, |ui| {
                                    conflict.ui(ui);
                                });
                            });
                        }
                    }

                    let is_safe_to_import =
                        self.profile_info_state.conflicts.iter().all(|c| c.resolved.is_some()) && !self.profile_info_state.mods_to_import.is_empty();

                    if is_safe_to_import {
                        ui.horizontal(|ui| {
                            ui.colored_label(Color32::GREEN, "✅");
                            ui.label("No conflicts found.");
                        });
                    }

                    if is_safe_to_import {
                        ui.collapsing("List of mods", |ui| {
                            egui::Grid::new("list_of_importing_mods").show(ui, |ui| {
                                for modification in self
                                    .profile_info_state
                                    .mods_to_import
                                    .iter()
                                    .chain(proceed_conflicting_mods(self.profile_info_state.conflicts.iter()))
                                {
                                    mod_info_ui(ui, modification);
                                    ui.end_row();
                                }
                            });
                        });
                    }

                    if ui.add_enabled(is_safe_to_import, egui::Button::new("Finish importing")).clicked() {
                        self.profile_info_state
                            .mods_to_import
                            .extend(proceed_conflicting_mods(self.profile_info_state.conflicts.iter()).cloned());

                        {
                            let mut lock = self.profile.write();
                            lock.mods.mods.extend(self.profile_info_state.mods_to_import.clone());
                            lock.mods.mods.sort();
                            lock.mods.mods.dedup_by(|a, b| a.project_id == b.project_id);
                        }

                        {
                            let id = self.profile.read().profile.id;
                            self.profiles.update_profile_config(id).report_error();
                        }

                        self.profile_info_state.mods_to_import.clear();
                        self.profile_info_state.conflicts.clear();
                        self.profile_info_state.mods_to_import_string.clear();
                    }
                });
            });

        egui::Window::new("Export mods")
            .open(&mut self.profile_info_state.is_export_window_open)
            .show(ui.ctx(), |ui| {
                let mods = &self.profile.read().mods.mods;

                egui::Grid::new("export_mods_selection").show(ui, |ui| {
                    if ui.button("Add all").clicked() {
                        self.profile_info_state.included_mods.iter_mut().for_each(|s| *s = true);
                    };

                    if ui.button("Remove all").clicked() {
                        self.profile_info_state.included_mods.iter_mut().for_each(|s| *s = false);
                    };

                    ui.end_row();

                    for (modification, state) in mods.iter().zip(self.profile_info_state.included_mods.iter_mut()) {
                        ui.checkbox(state, "");
                        mod_info_ui(ui, modification);
                        ui.end_row();
                    }
                });

                if ui
                    .add_enabled(
                        !self.profile_info_state.included_mods.iter().all(|s| !s),
                        egui::Button::new("Finish exporting"),
                    )
                    .on_hover_text("The export code will include all the mods that have the checkbox checked")
                    .clicked()
                {
                    let mods = mods
                        .iter()
                        .zip(self.profile_info_state.included_mods.iter())
                        .filter(|(_, s)| **s)
                        .map(|(m, _)| m.clone())
                        .map(|mut m| {
                            m.is_downloaded = false;
                            m
                        })
                        .collect::<Vec<_>>();

                    if let Some(export_code) = serde_json::to_string(&mods).report_error() {
                        ui.ctx().copy_text(export_code);
                    }

                    toasts::add(|toasts| toasts.success("Copied the export code to the clipboard"));
                }
            });

        ui.heading("Profile");

        ui.small("Don't forget to save the changes!");

        ui.label("Profile name");
        TextEdit::singleline(&mut self.profile_info_state.profile_name).show(ui);

        ui.label("JVM arguments");

        ui.small("Each element should represent only one argument.");

        ui.vertical(|ui| {
            egui::Grid::new("jvm_arguments_ui").show(ui, |ui| {
                self.profile_info_state.profile_jvm_args.retain_mut(|i| {
                    ui.scope(|ui| {
                        ui.set_min_width(150.0);
                        ui.text_edit_singleline(i);
                    });
                    let response = ui.button("❌");
                    ui.end_row();
                    *i = i.trim().to_string();
                    !response.clicked()
                });
            });

            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.profile_info_state.jvm_arg_to_add);
                if ui.button("Add").clicked() {
                    let value = std::mem::take(&mut self.profile_info_state.jvm_arg_to_add).trim().to_string();
                    self.profile_info_state.profile_jvm_args.push(value)
                };
            });
        });

        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                {
                    let mut profile = self.profile.write();
                    profile.profile.name.clone_from(&self.profile_info_state.profile_name);
                    if let ProfileState::Downloaded(instance) = &mut profile.profile.state {
                        instance.jvm_arguments_mut().clone_from(&self.profile_info_state.profile_jvm_args);
                    }

                    if let Some(instance) = self.profiles.find_instance(profile.profile.id.instance()) {
                        if let Some(profile) = instance.write().find_profile_mut(profile.profile.id) {
                            profile.name.clone_from(&self.profile_info_state.profile_name);
                        }
                    }
                }

                self.profiles
                    .update_instance_config(self.profile.read().profile.id.instance())
                    .report_error();
                self.profiles.update_profile_config(self.profile.read().profile.id).report_error();
            }

            if ui.button("Reset").clicked() {
                self.profile_info_state.set_profile_to_edit(&self.profile.read());
            }
        });

        ui.heading("Mods");

        ui.add_enabled_ui(self.profile.read().profile.loader().support_mods(), |ui| {
            ui.toggle_button(&mut self.profile_info_state.is_import_window_open, "Import mods");
            if ui
                .toggle_button(&mut self.profile_info_state.is_export_window_open, "Export mods")
                .clicked()
            {
                self.profile_info_state.included_mods = vec![true; self.profile.read().mods.mods.len()];
            }

            if ui
                .button("Open mods folder")
                .on_hover_text("Open a folder where mods for this profile are located.")
                .clicked()
            {
                let profile_id = self.profile.read().profile.id;
                let path = mods_stash_path_for_profile(profile_id);

                if !path.exists() {
                    std::fs::create_dir_all(&path).report_error();
                }
                if let Ok(path) = std::fs::canonicalize(path) {
                    open_directory_native(path).report_error();
                }
            }
        });

        if ui.button("Browse mods").clicked() {
            let kind = TabKind::Mods {
                profile: self.profile.clone(),
            };
            self.tabs_state.0.insert(kind.id(), kind);
        }

        egui::ScrollArea::vertical().min_scrolled_width(ui.available_width()).show(ui, |ui| {
            let (mut vec, profile_id) = {
                let profile = &mut self.profile.write();
                (std::mem::take(&mut profile.mods.mods), profile.profile.id)
            };
            let mut mods_to_remove = Vec::new();
            egui::Grid::new("mods_list").show(ui, |ui| {
                for m in &mut vec {
                    mod_info_ui(ui, m);

                    if self.profile_info_state.currently_downloading_mods.contains(&m.project_id) {
                        ui.spinner();
                        ui.label("Downloading...");
                    } else if m.is_downloaded && !self.profile_info_state.currently_downloading_mods.contains(&m.project_id) {
                        ui.colored_label(Color32::GREEN, "Downloaded");
                        ui.button_with_confirm_popup(Id::new(&m.version_id).with("delete"), "Delete", |ui| {
                            ui.label("Are you sure you want to delete this mod?");
                            ui.horizontal(|ui| {
                                let yes = ui.button("Yes");
                                let no = ui.button("No");

                                if yes.clicked() {
                                    mods_to_remove.push(m.project_id.clone());
                                    let path = mods_stash_path_for_profile(profile_id);
                                    for file in &m.files {
                                        std::fs::remove_file(path.join(&file.filename)).report_error();
                                    }
                                }

                                if yes.clicked() || no.clicked() {
                                    ui.memory_mut(|mem| mem.close_popup());
                                }
                            });
                        });
                    } else {
                        let profile_id = self.profile.read().profile.id;
                        let files = m.files.clone();
                        let project_id = m.project_id.clone();
                        let ctx = ui.ctx().clone();
                        let download_task = Task::new(
                            "Download mod",
                            Caller::progressing(move |progress| async move {
                                download_added_mod(progress, ctx, mods_stash_path_for_profile(profile_id), files).await;
                                (profile_id, project_id)
                            }),
                        );

                        if ui.button("Download").clicked() {
                            self.task_manager.push_task::<DownloadAddedModsCollection>(download_task);
                            m.is_downloaded = true
                        }
                    }

                    ui.end_row()
                }
            });

            vec.retain(|m| !mods_to_remove.contains(&m.project_id));
            if !mods_to_remove.is_empty() {
                self.profiles.update_profile_config(self.profile.read().profile.id).report_error();
            }

            let _ = std::mem::replace(&mut self.profile.write().mods.mods, vec);
        });
    }
}
