use std::{path::Path, sync::Arc};

use eframe::egui::Button;

use crate::{errors_pool::ErrorPoolExt, open_directory::open_directory_native, TabKind, DOT_NOMI_MODS_STASH_DIR};

use super::{ModdedProfile, TabsState, View};

pub struct ProfileInfo<'a> {
    pub profile: &'a Arc<ModdedProfile>,
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
        ui.heading("Mods");

        ui.vertical(|ui| {
            for m in &self.profile.mods.mods {
                ui.label(&m.name);
            }
        });

        if ui
            .add_enabled(self.profile.profile.loader().is_fabric(), Button::new("Open mods folder"))
            .on_hover_text("Open a folder where mods for this profile are located.")
            .on_hover_text(
                "You can add your own mods but they will not be shown in the list above.\nAlthough they still will be loaded automatically.",
            )
            .clicked()
        {
            let path = Path::new(DOT_NOMI_MODS_STASH_DIR).join(format!("{}", self.profile.profile.id));
            if !path.exists() {
                std::fs::create_dir_all(&path).report_error();
            }
            if let Ok(path) = std::fs::canonicalize(path) {
                open_directory_native(path).report_error();
            }
        }

        if ui
            .add_enabled(self.profile.profile.loader().is_fabric(), Button::new("Browse mods"))
            .on_disabled_hover_text("Profile must have a mod loader. For example Fabric")
            .clicked()
        {
            self.tabs_state.0.push(TabKind::Mods {
                profile: self.profile.clone(),
            });
        }
    }
}
