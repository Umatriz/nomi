// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// pub mod bootstrap;
// pub mod configs;
// pub mod downloads;
// pub mod loaders;
// pub mod manifest;
// pub mod profiles;
// pub mod utils;

pub mod error;
pub mod prelude;
pub mod utils;

use std::path::PathBuf;

use once_cell::sync::Lazy;

pub static HOME: Lazy<PathBuf> = Lazy::new(|| std::env::current_dir().unwrap_or_default());
pub static LOGS: Lazy<PathBuf> = Lazy::new(|| HOME.join("logs"));
// TODO: config.toml
pub static CONFIG: Lazy<PathBuf> = Lazy::new(|| HOME.join("config.json"));

pub static GAME: Lazy<PathBuf> = Lazy::new(|| HOME.join("minecraft"));
pub static GAME_LIBRARIES: Lazy<PathBuf> = Lazy::new(|| GAME.join("libraries"));
pub static GAME_VERSIONS: Lazy<PathBuf> = Lazy::new(|| GAME.join("versions"));

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
