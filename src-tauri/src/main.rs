// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;

use commands::{
  download_version,
  launch
};

use utils::{Config, Profile};

#[tokio::main]
async fn main() {
  // let conf = Config::new("username".to_string());
  // let prof = Profile::new("1.19.4".to_string(), "release".to_string(), "E\\mine".to_string(), conf.profiles);

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      download_version,
      launch
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}