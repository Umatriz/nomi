use std::sync::Arc;

use eframe::egui;
use nomi_core::instance::logs::GameLogsWriter;
use parking_lot::Mutex;

use crate::subscriber::EguiLayer;

use super::View;

pub struct Logs<'a> {
    pub egui_layer: &'a EguiLayer,
    pub logs_state: &'a mut LogsState,
}

#[derive(Default)]
pub struct LogsState {
    pub selected_tab: LogsPage,
    pub game_logs: Arc<GameLogs>,
}

#[derive(Default, PartialEq)]
pub enum LogsPage {
    #[default]
    Game,
    Launcher,
}

impl LogsState {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }
}

impl View for Logs<'_> {
    fn ui(mut self, ui: &mut eframe::egui::Ui) {
        egui::TopBottomPanel::top("logs_page_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.logs_state.selected_tab, LogsPage::Game, "Game");
                ui.selectable_value(&mut self.logs_state.selected_tab, LogsPage::Launcher, "Launcher");
            });
        });

        match self.logs_state.selected_tab {
            LogsPage::Game => self.game_ui(ui),
            LogsPage::Launcher => self.launcher_ui(ui),
        }
    }
}

impl Logs<'_> {
    pub fn game_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both().stick_to_bottom(true).show(ui, |ui| {
            ui.vertical(|ui| {
                let lock = self.logs_state.game_logs.logs.lock();
                for message in lock.iter() {
                    ui.label(message);
                }
            });
        });
    }

    pub fn launcher_ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both().stick_to_bottom(true).show(ui, |ui| self.egui_layer.ui(ui));
    }
}

#[derive(Default)]
pub struct GameLogs {
    logs: Arc<Mutex<Vec<String>>>,
}

impl GameLogs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&self) {
        self.logs.lock().clear();
    }
}

impl GameLogsWriter for GameLogs {
    fn write(&self, data: nomi_core::instance::logs::GameLogsEvent) {
        self.logs.lock().push(data.into_message());
    }
}
