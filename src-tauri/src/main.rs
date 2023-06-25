#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;
pub mod configs;

use eframe::egui::{self, CentralPanel, Frame};
use egui_dock::{Style, DockArea};

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}