use std::{path::Path, sync::Arc};

use eframe::egui::Button;
use egui_task_manager::TaskManager;
use parking_lot::RwLock;

use crate::{errors_pool::ErrorPoolExt, open_directory::open_directory_native, TabKind, DOT_NOMI_MODS_STASH_DIR};

use super::{ModdedProfile, TabsState, View};

pub struct ProfileInfo<'a> {
    pub task_manager: &'a mut TaskManager,
    pub profile: Arc<RwLock<ModdedProfile>>,
    pub tabs_state: &'a mut TabsState,
    pub profile_info_state: &'a mut ProfileInfoState,
}

#[derive(Default)]
pub struct ProfileInfoState {
    // pub current_profile: VersionProfile,
}

impl ProfileInfoState {
    pub fn new() -> Self {
        Self {}
    }
}

impl View for ProfileInfo<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        let profile = self.profile.read();

        ui.heading("Mods");

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                for m in &profile.mods.mods {
                    ui.label(&m.name);
                    // if m.is_downloaded {
                    //     ui.colored_label(Color32::GREEN, "Downloaded");
                    // }else if  {

                    // } else {
                    //     let profile_id = self.profile.profile.id;
                    //     let files = m.files.clone();
                    //     let download_task = Task::new(
                    //         "Download mod",
                    //         Caller::progressing(move |progress| download_added_mod(progress, profile_id, files)),
                    //     );

                    //     if ui.button("Download").clicked() {
                    //         self.task_manager.push_task::<DownloadAddedModsCollection>(download_task);
                    //         self.profile
                    //     }
                    // }
                }
            });
        });

        if ui
            .add_enabled(profile.profile.loader().is_fabric(), Button::new("Open mods folder"))
            .on_hover_text("Open a folder where mods for this profile are located.")
            .on_hover_text(
                "You can add your own mods but they will not be shown in the list above.\nAlthough they still will be loaded automatically.",
            )
            .clicked()
        {
            let path = Path::new(DOT_NOMI_MODS_STASH_DIR).join(format!("{}", profile.profile.id));
            if !path.exists() {
                std::fs::create_dir_all(&path).report_error();
            }
            if let Ok(path) = std::fs::canonicalize(path) {
                open_directory_native(path).report_error();
            }
        }

        if ui
            .add_enabled(profile.profile.loader().is_fabric(), Button::new("Browse mods"))
            .on_disabled_hover_text("Profile must have a mod loader. For example Fabric")
            .clicked()
        {
            let kind = TabKind::Mods {
                profile: self.profile.clone(),
            };
            self.tabs_state.0.insert(kind.id(), kind);
        }
    }
}
