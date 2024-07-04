use eframe::egui::Ui;

pub mod add_profile_menu;
pub mod add_tab_menu;
pub mod profiles;
pub mod settings;
pub mod tasks_manager;

pub use add_profile_menu::*;
pub use add_tab_menu::*;
pub use profiles::*;
pub use settings::*;
pub use tasks_manager::*;

pub trait Component: Sized {
    fn ui(self, ui: &mut Ui);
}
