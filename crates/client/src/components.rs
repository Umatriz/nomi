use eframe::egui::Ui;

pub mod add_profile_menu;
pub mod add_tab_menu;
pub mod profiles;
pub mod settings;
pub mod tasks_manager;

pub trait Component: Sized {
    fn ui(self, ui: &mut Ui);
}
