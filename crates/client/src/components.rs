use eframe::egui::Ui;

use crate::Storage;

pub mod add_profile_menu;
pub mod add_tab_menu;
pub mod profiles;

pub trait Component: Sized {
    fn ui(self, ui: &mut Ui);
}

pub trait StorageCreationExt {
    fn extend(storage: &mut Storage) -> anyhow::Result<()>;
}
