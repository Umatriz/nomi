use eframe::egui::Ui;

pub mod add_profile_menu;
pub mod add_tab_menu;
pub mod downloading_progress;
pub mod profiles;
pub mod settings;

pub use add_profile_menu::*;
pub use add_tab_menu::*;
pub use downloading_progress::*;
pub use profiles::*;
pub use settings::*;

pub trait Component: Sized {
    fn ui(self, ui: &mut Ui);
}
