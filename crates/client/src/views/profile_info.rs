use std::sync::Arc;

use nomi_core::configs::profile::VersionProfile;

use crate::TabKind;

use super::{ModdedProfile, SimpleProfile, TabsState, View};

pub struct ProfileInfo<'a> {
    pub profile: Arc<ModdedProfile>,
    pub tabs_state: &'a mut TabsState,
    pub profile_info_state: &'a mut ProfileInfoState,
}

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
        if ui.button("Browse mod").clicked() {
            self.tabs_state.0.insert(TabKind::Mods {
                profile: self.profile.clone(),
            });
        }
    }
}
