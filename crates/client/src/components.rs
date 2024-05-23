use eframe::egui::Ui;

use crate::Storage;

pub mod profiles;

pub trait Component: Sized {
    fn ui(self, ui: &mut Ui, storage: &mut Storage);
}

pub trait StorageCreationExt {
    fn extend(storage: &mut Storage) -> anyhow::Result<()>;
}
