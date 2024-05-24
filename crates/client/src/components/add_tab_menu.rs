use egui_dock::DockState;

use crate::Tab;

use super::Component;

pub struct AddTab<'a> {
    pub dock_state: &'a DockState<Tab>,
    pub added_tabs: &'a mut Vec<Tab>,
}

impl Component for AddTab<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        let opened_tabs = self
            .dock_state
            .iter_all_tabs()
            .map(|(_, tab)| tab.clone())
            .collect::<Vec<_>>();

        ui.menu_button("View", |ui| {
            let unopened_tabs = Tab::ALL_TABS
                .iter()
                .filter(|tab| !opened_tabs.contains(tab))
                .collect::<Vec<_>>();

            match unopened_tabs.len() {
                0 => {
                    ui.label("All tabs are already open");
                }
                _ => {
                    for unopened_tab in unopened_tabs {
                        if ui.button(unopened_tab.as_str()).clicked() {
                            self.added_tabs.push(unopened_tab.clone());
                        }
                    }
                }
            }
        });

        // egui::popup_below_widget(ui, popup_id, &button_response, |ui| {
        //     ui.set_min_width(200.0);
        //     ui.vertical(|ui| {
        //         ui.label("Unopened tabs");
        //     });
        // });
    }
}
