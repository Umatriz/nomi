use eframe::egui::{self, Layout};
use egui_task_manager::TaskManager;

use super::{profiles::InstancesState, View};

pub struct DownloadingProgress<'a> {
    pub manager: &'a TaskManager,
    pub profiles_state: &'a mut InstancesState,
}

impl View for DownloadingProgress<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            for collection in self.manager.iter_collections() {
                for task in collection.iter_tasks() {
                    ui.group(|ui| task.ui(ui));
                }
            }
        });
    }
}
