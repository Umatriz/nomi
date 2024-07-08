use eframe::egui::Ui;

pub mod add_profile_menu;
pub mod add_tab_menu;
pub mod downloading_progress;
pub mod mods_manager;
pub mod profiles;
pub mod settings;

pub use add_profile_menu::*;
pub use add_tab_menu::*;
pub use downloading_progress::*;
pub use mods_manager::*;
pub use profiles::*;
pub use settings::*;

pub trait View: Sized {
    fn ui(self, ui: &mut Ui);
}
