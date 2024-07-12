use std::{sync::Arc};

use egui_dock::DockState;

use crate::TabKind;

use super::{ModdedProfile, ProfilesConfig, View};

pub struct AddTab<'a> {
    pub dock_state: &'a DockState<TabKind>,
    pub tabs_state: &'a mut TabsState,
}

#[derive(Default)]
pub struct TabsState(pub Vec<TabKind>);

impl TabsState {
    pub fn new() -> Self {
        let tabs = vec![TabKind::Profiles, TabKind::Logs, TabKind::Settings, TabKind::DownloadProgress];
        Self(tabs)
    }

    pub fn update_profile_tabs(&mut self, dock_state: &mut DockState<TabKind>, profiles: &ProfilesConfig, old: Arc<ModdedProfile>) {
        // PANICS: Will never panic since the tab cannot be opened if the profile does not exists
        let prof = profiles.find_profile(old.profile.id).unwrap();

        let mut iter = self.0.iter_mut();

        let mut tabs_iter = dock_state.iter_all_tabs_mut().map(|t| t.1);

        let target = TabKind::ProfileInfo { profile: old.clone() };
        if let Some((TabKind::ProfileInfo { profile }, TabKind::ProfileInfo { profile: dock_profile })) = iter
            .find(|t| *t == &target)
            .and_then(|t| tabs_iter.find(|tab| *tab == &target).map(|dock_tab| (t, dock_tab)))
        {
            *profile = prof.clone();
            *dock_profile = prof.clone();
        }

        let target = TabKind::Mods { profile: old.clone() };
        if let Some((TabKind::Mods { profile }, TabKind::Mods { profile: dock_profile })) = iter
            .find(|t| *t == &target)
            .and_then(|t| tabs_iter.find(|tab| *tab == &target).map(|dock_tab| (t, dock_tab)))
        {
            *profile = prof.clone();
            *dock_profile = prof.clone();
        }
    }
}

impl View for AddTab<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        ui.menu_button("View", |ui| {
            let tabs_state = &mut self.tabs_state.0;
            for tab in TabKind::AVAILABLE_TABS_TO_OPEN {
                let mut is_open = tabs_state.contains(tab);
                ui.toggle_value(&mut is_open, tab.name());

                if is_open {
                    if !tabs_state.contains(tab) {
                        tabs_state.push(tab.to_owned());
                    }
                } else if let Some(index) = tabs_state.iter().position(|t| t == tab) {
                    tabs_state.remove(index);
                }
            }
        })
        .response
        .on_hover_text("Add additional tabs");
    }
}
