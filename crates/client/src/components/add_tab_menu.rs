use std::collections::HashSet;

use egui_dock::DockState;

use crate::{Tab, TabId, TabKind};

use super::Component;

pub struct AddTab<'a> {
    pub dock_state: &'a DockState<Tab>,
    pub tabs_state: &'a mut TabsState,
}

pub struct TabsState(pub HashSet<TabId>);

fn set_open(open: &mut HashSet<TabId>, key: &TabId, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}

impl Component for AddTab<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        ui.menu_button("View", |ui| {
            let tabs_state = &mut self.tabs_state.0;
            for tab in TabKind::AVAILABLE_TABS_TO_OPEN {
                let mut is_open = tabs_state.contains(&tab.id());
                ui.toggle_value(&mut is_open, tab.name());
                set_open(tabs_state, &tab.id(), is_open)
            }
        });
    }
}
