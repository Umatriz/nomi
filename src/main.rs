#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;
pub mod ui;

use commands::{
  download_version,
  launch,
  get_manifest
};

use downloads::launcher_manifest::{LauncherManifestVersion, LauncherManifest};
use eframe::egui;
use tokio::runtime::Builder;

fn main() -> Result<(), eframe::Error> {
  let options = eframe::NativeOptions {
      initial_window_size: Some(egui::vec2(320.0, 240.0)),
      ..Default::default()
  };
  eframe::run_native(
      "My egui App",
      options,
      Box::new(|_cc| Box::<MyApp>::default()),
  )
}

struct MyApp {
  name: String,
  age: u32,
  versions: Vec<LauncherManifestVersion>
}

impl Default for MyApp {
  fn default() -> Self {
      let runtime = Builder::new_multi_thread()
          .worker_threads(1)
          .enable_all()
          .build()
          .unwrap();
      Self {
          name: "Arthur".to_owned(),
          age: 42,
          versions: runtime.block_on(get_manifest()).unwrap()
      }
  }
}

impl eframe::App for MyApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      egui::CentralPanel::default().show(ctx, |ui| {
          ui.heading("My egui Application");
          ui.horizontal(|ui| {
              let name_label = ui.label("Your name: ");
              ui.text_edit_singleline(&mut self.name)
                  .labelled_by(name_label.id);
          });
          ui.vertical(|ui| {
              for v in self.versions.iter() {
                  ui.label(format!("{:#?}", v));
              }
          });
          ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
          if ui.button("Click each year").clicked() {
              self.age += 1;
          }
          ui.label(format!("Hello '{}', age {}", self.name, self.age));
      });
  }
}