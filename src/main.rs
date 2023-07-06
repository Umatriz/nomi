pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;
pub mod configs;
pub mod loaders;

use commands::{download_version};

slint::include_modules!();
#[tokio::main]
async fn main() {
  let ui = MainWindow::new().unwrap();
  ui.global::<State>().on_launch(|_id| {
    tokio::spawn(download_version("id".to_string()));
  });
  ui.run().unwrap();
}