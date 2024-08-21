use egui_task_manager::TaskManager;

use super::{profiles::InstancesState, View};

pub struct DownloadingProgress<'a> {
    pub manager: &'a TaskManager,
    pub profiles_state: &'a mut InstancesState,
}

impl View for DownloadingProgress<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        ui.vertical(|ui| {
            for collection in self.manager.iter_collections() {
                for task in collection.iter_tasks() {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        task.ui(ui)
                    });
                }
            }
        });
    }
}
