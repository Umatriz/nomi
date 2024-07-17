use std::collections::HashMap;

use egui_dock::DockState;

use crate::{Tab, TabId, TabKind};

use super::{ModdedProfile, View};

pub struct AddTab<'a> {
    pub dock_state: &'a DockState<Tab>,
    pub tabs_state: &'a mut TabsState,
}

#[derive(Default)]
pub struct TabsState(pub HashMap<TabId, TabKind>);

impl TabsState {
    pub fn new() -> Self {
        let tabs = [TabKind::Profiles, TabKind::Logs, TabKind::Settings, TabKind::DownloadProgress]
            .map(|t| (t.id(), t))
            .into_iter()
            .collect();
        Self(tabs)
    }

    pub fn remove_profile_related_tabs(&mut self, profile_to_remove: &ModdedProfile) {
        self.0.retain(|_id, k| match k {
            TabKind::Mods { profile } => *profile.read() != *profile_to_remove,
            TabKind::ProfileInfo { profile } => *profile.read() != *profile_to_remove,
            _ => true,
        });
    }
}

impl View for AddTab<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        ui.menu_button("View", |ui| {
            let tabs_state = &mut self.tabs_state.0;
            for (id, tab) in TabKind::AVAILABLE_TABS_TO_OPEN.iter().map(|kind| (kind.id(), kind)) {
                let mut is_open = tabs_state.contains_key(&id);
                ui.toggle_value(&mut is_open, &*tab.id());

                if is_open {
                    tabs_state.entry(id).or_insert_with(|| tab.to_owned());
                } else {
                    tabs_state.remove(&id);
                }
            }
        })
        .response
        .on_hover_text("Add additional tabs");
    }
}
