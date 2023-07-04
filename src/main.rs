pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;
pub mod configs;
pub mod loaders;

use commands::{download_version, get_manifest, get_config, launch};

slint::include_modules!();
fn main() {
  let ui = MainWindow::new().unwrap();
  ui.global::<State>().on_launch(|id| {
    println!("id: {}", id);
  });
  ui.run().unwrap();
}