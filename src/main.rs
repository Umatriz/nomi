#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;
pub mod configs;
pub mod ui;

use eframe::egui::{self, CentralPanel, Frame};
use egui_dock::{Style, DockArea};
use ui::{Main};


fn main() -> Result<(), eframe::Error> {
  let options = eframe::NativeOptions {
    initial_window_size: Some(egui::vec2(1280.0, 720.0)),
      ..Default::default()
  };
  eframe::run_native(
    "My egui App",
    options,
    Box::new(|_cc| Box::<Main>::default()),
  )
}

impl eframe::App for Main {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    CentralPanel::default()
    // When displaying a DockArea in another UI, it looks better
    // to set inner margins to 0.
      .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
      .show(ctx, |ui| {
        let style = self
          .context
          .style
          .get_or_insert(Style::from_egui(ui.style()))
          .clone();

        DockArea::new(&mut self.tree)
          .style(style)
          .show_close_buttons(self.context.show_close_buttons)
          .show_add_buttons(self.context.show_add_buttons)
          .draggable_tabs(self.context.draggable_tabs)
          .show_tab_name_on_hover(self.context.show_tab_name_on_hover)
          .show_inside(ui, &mut self.context);
    });
  }
}