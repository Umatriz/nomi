use std::{collections::HashSet, hash::Hash};

use egui_dock::DockState;

use crate::{set_selected, Tab, TabId, TabKind};

use super::View;

pub struct AddTab<'a> {
    pub dock_state: &'a DockState<Tab>,
    pub tabs_state: &'a mut TabsState,
}

#[derive(Default)]
pub struct TabsState(pub HashSet<TabId>);

impl TabsState {
    pub fn new() -> Self {
        let mut tabs = HashSet::new();

        tabs.insert(TabId::PROFILES);
        tabs.insert(TabId::MODS);
        tabs.insert(TabId::LOGS);
        tabs.insert(TabId::SETTINGS);
        tabs.insert(TabId::DOWNLOAD_PROGRESS);

        Self(tabs)
    }
}

impl View for AddTab<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        ui.menu_button("View", |ui| {
            let tabs_state = &mut self.tabs_state.0;
            for tab in TabKind::AVAILABLE_TABS_TO_OPEN {
                let mut is_open = tabs_state.contains(&tab.id());
                ui.toggle_value(&mut is_open, tab.name());
                set_selected(tabs_state, &tab.id(), is_open)
            }
        })
        .response
        .on_hover_text("Add additional tabs");
    }
}
