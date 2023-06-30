#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;
pub mod configs;

use commands::{download_version, get_manifest, get_config};

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      download_version,
      get_manifest,
      get_config
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}