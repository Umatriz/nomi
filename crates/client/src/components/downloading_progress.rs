use eframe::egui::{self, Layout};
use egui_task_manager::TaskManager;

use super::{profiles::ProfilesState, Component};

pub struct DownloadingProgress<'a> {
    pub manager: &'a TaskManager,
    pub profiles_state: &'a mut ProfilesState,
}

impl Component for DownloadingProgress<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        ui.with_layout(Layout::top_down_justified(egui::Align::Min), |ui| {
            self.manager.ui(ui)
        });
    }
}
