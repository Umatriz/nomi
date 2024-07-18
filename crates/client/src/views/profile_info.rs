use std::{collections::HashSet, path::Path, sync::Arc};

use eframe::egui::{self, Color32, Id, RichText};
use egui_task_manager::{Caller, Task, TaskManager};
use nomi_modding::modrinth::project::ProjectId;
use parking_lot::RwLock;

use crate::{
    collections::DownloadAddedModsCollection, errors_pool::ErrorPoolExt, open_directory::open_directory_native, ui_ext::UiExt, views::ProfilesConfig,
    TabKind, DOT_NOMI_MODS_STASH_DIR,
};

use super::{download_added_mod, Mod, ModdedProfile, TabsState, View};

pub struct ProfileInfo<'a> {
    pub profiles: &'a ProfilesConfig,
    pub task_manager: &'a mut TaskManager,
    pub profile: Arc<RwLock<ModdedProfile>>,
    pub tabs_state: &'a mut TabsState,
    pub profile_info_state: &'a mut ProfileInfoState,
}

#[derive(Default)]
pub struct ProfileInfoState {
    pub currently_downloading_mods: HashSet<ProjectId>,

    pub is_import_window_open: bool,
    pub mods_to_import_string: String,
    pub mods_to_import: Vec<Mod>,
    pub conflicts: Vec<ImportConflict>,
}

pub struct ImportConflict {
    pub name: String,
    pub existing: Mod,
    pub incoming: Mod,
    pub resolved: Option<ImportConflictSolution>,
}

impl ImportConflict {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        fn show_variant(ui: &mut egui::Ui, conflict_resolved: &mut Option<ImportConflictSolution>, variant: &Mod, grid_id: impl std::hash::Hash) {
            ui.vertical(|ui| {
                egui::Grid::new(Id::new(grid_id).with(ui.id())).show(ui, |ui| {
                    ui.label("Name:");
                    ui.label(&variant.name);
                    ui.end_row();

                    ui.label("Version number:");
                    ui.label(&variant.version_number);
                    ui.end_row();

                    ui.label("Version name:");
                    ui.label(&variant.version_name);
                    ui.end_row();

                    ui.label("Files:");
                    ui.vertical(|ui| {
                        for file in &variant.files {
                            ui.label(&file.filename);
                        }
                    });
                });

                ui.button_with_confirm_popup("Accept", |ui| {
                    ui.label("Are you sure you wanna accept this variant to solve the conflict?");
                    ui.horizontal(|ui| {
                        let yes_button = ui.button("Yes");
                        let no_button = ui.button("No");

                        if ui.button("Yes").clicked() {
                            *conflict_resolved = Some(ImportConflictSolution::Existing)
                        };

                        if yes_button.clicked() || no_button.clicked() {
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                    });
                })
            });
        }

        ui.horizontal(|ui| {
            show_variant(ui, &mut self.resolved, &self.existing, "conflict_existing_id");
            ui.separator();
            show_variant(ui, &mut self.resolved, &self.incoming, "conflict_incoming_id");
        });
    }
}

pub enum ImportConflictSolution {
    Existing,
    Incoming,
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

                    if ui.add_enabled(is_safe_to_import, egui::Button::new("Finish importing")).clicked() {
                        self.profile_info_state
                            .mods_to_import
                            .extend(self.profile_info_state.conflicts.iter().filter_map(|c| {
                                c.resolved.as_ref().map(|resolved| match resolved {
                                    ImportConflictSolution::Existing => c.existing.clone(),
                                    ImportConflictSolution::Incoming => c.incoming.clone(),
                                })
                            }));

                        {
                            let mut lock = self.profile.write();
                            lock.mods.mods.extend(self.profile_info_state.mods_to_import.clone());
                            lock.mods.mods.sort();
                            lock.mods.mods.dedup_by(|a, b| a.project_id == b.project_id);
                        }

                        {
                            self.profiles.update_config().report_error();
                        }

                        self.profile_info_state.mods_to_import.clear();
                        self.profile_info_state.conflicts.clear();
                        self.profile_info_state.mods_to_import_string.clear();
                    }
                });
            });

        ui.heading("Mods");

        ui.add_enabled_ui(self.profile.read().profile.loader().is_fabric(), |ui| {
            ui.toggle_button(&mut self.profile_info_state.is_import_window_open, "Import mods");

            if ui.button("Export mods").clicked() {
                let mods = self
                    .profile
                    .read()
                    .mods
                    .mods
                    .iter()
                    .cloned()
                    .map(|mut m| {
                        m.is_downloaded = false;
                        m
                    })
                    .collect::<Vec<_>>();

                if let Some(export_code) = serde_json::to_string(&mods).report_error() {
                    ui.output_mut(|o| o.copied_text = export_code);
                }

                ui.toasts(|toasts| toasts.success("Copied the export code to the clipboard"));
            };

            if ui
                .button("Open mods folder")
                .on_hover_text("Open a folder where mods for this profile are located.")
                .clicked()
            {
                let path = Path::new(DOT_NOMI_MODS_STASH_DIR).join(format!("{}", self.profile.read().profile.id));
                if !path.exists() {
                    std::fs::create_dir_all(&path).report_error();
                }
                if let Ok(path) = std::fs::canonicalize(path) {
                    open_directory_native(path).report_error();
                }
            }

            if ui
                .button("Browse mods")
                .on_disabled_hover_text("Profile must have a mod loader.")
                .clicked()
            {
                let kind = TabKind::Mods {
                    profile: self.profile.clone(),
                };
                self.tabs_state.0.insert(kind.id(), kind);
            }
        });

        egui::ScrollArea::vertical().min_scrolled_width(ui.available_width()).show(ui, |ui| {
            let mut vec = std::mem::take(&mut self.profile.write().mods.mods);
            for m in &mut vec {
                ui.horizontal(|ui| {
                    ui.label(&m.name);
                    if self.profile_info_state.currently_downloading_mods.contains(&m.project_id) {
                        ui.spinner();
                        ui.label("Downloading...");
                    } else if m.is_downloaded {
                        ui.colored_label(Color32::GREEN, "Downloaded");
                    } else {
                        let profile_id = self.profile.read().profile.id;
                        let files = m.files.clone();
                        let project_id = m.project_id.clone();
                        let download_task = Task::new(
                            "Download mod",
                            Caller::progressing(move |progress| async move {
                                download_added_mod(progress, profile_id, files).await;
                                project_id
                            }),
                        );

                        if ui.button("Download").clicked() {
                            self.task_manager.push_task::<DownloadAddedModsCollection>(download_task);
                            m.is_downloaded = true
                        }
                    }
                });
            }

            let _ = std::mem::replace(&mut self.profile.write().mods.mods, vec);
        });
    }
}
