// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;

use commands::{
  download_version,
  launch,
  get_manifest
};

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      download_version,
      launch,
      get_manifest
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}