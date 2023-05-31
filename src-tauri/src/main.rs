// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;

use downloads::Download;
use std::env;

#[tokio::main]
async fn main() {
  // let dir = env::current_dir().unwrap();
  // let target = format!("{}{}", dir.to_str().unwrap().to_string(), "\\minecraft");

  // TEST CODE!
  let load = Download::new();
  load.await.download("1.19.4".to_string(), "E:\\programming\\code\\nomi".to_string()).await.unwrap();

  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}