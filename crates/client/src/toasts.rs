use std::sync::{Arc, LazyLock};

use eframe::egui::Context;
use egui_notify::{Toast, Toasts};
use parking_lot::RwLock;

pub static TOASTS: LazyLock<Arc<RwLock<Toasts>>> = LazyLock::new(|| Arc::new(RwLock::new(new())));

fn new() -> Toasts {
    Toasts::default()
}

pub fn show(ctx: &Context) {
    TOASTS.write().show(ctx)
}

pub fn add(writer: impl FnOnce(&mut Toasts) -> &mut Toast) {
    let mut toasts = TOASTS.write();
    writer(&mut toasts);
}
