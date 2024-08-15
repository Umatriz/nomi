use eframe::egui;
use nomi_core::instance::Instance;

use crate::{errors_pool::ErrorPoolExt, toasts, ui_ext::UiExt};

use super::{InstancesState, View};

pub struct CreateInstanceMenu<'a> {
    pub instances_state: &'a mut InstancesState,
    pub create_instance_menu_state: &'a mut CreateInstanceMenuState,
}

#[derive(Default)]
pub struct CreateInstanceMenuState {
    pub name: String,
}

impl CreateInstanceMenuState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl View for CreateInstanceMenu<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        egui::TextEdit::singleline(&mut self.create_instance_menu_state.name)
            .hint_text("Instance name")
            .show(ui);

        if ui
            .add_enabled(!self.create_instance_menu_state.name.trim_end().is_empty(), egui::Button::new("Create"))
            .clicked()
        {
            let id = self.instances_state.instances.next_id();
            let instance = Instance::new(self.create_instance_menu_state.name.trim_end().to_owned(), id);
            self.instances_state.instances.add_instance(instance);
            self.instances_state.instances.update_instance_config(id).report_error();
            toasts::add(|toasts| toasts.success("New instance created"));
        }
    }
}
