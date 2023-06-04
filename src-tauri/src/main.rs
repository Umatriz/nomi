// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;

use bootstrap::{Version, VersionType};
use downloads::Download;
use std::{env, path::Path};

#[tokio::main]
async fn main() {
  // let dir = env::current_dir().unwrap();
  // let target = format!("{}{}", dir.to_str().unwrap().to_string(), "\\minecraft");

  // TEST CODE!
  // let load = Download::new();
  // load.await.download("1.19.4".to_string(), "E:\\programming\\code\\nomi\\minecraft".to_string()).await.unwrap();

  // let asset = downloads::assets::AssetsDownload::new("https://piston-meta.mojang.com/v1/packages/b7d40993905fa101bc0884ccd883bd835c51ac13/3.json".to_string(), "3".to_string());
  // match asset.await.get_assets_json(Path::new("E:\\programming\\code\\nomi\\minecraft").join("assets").join("indexes")).await {
  //   Ok(_) => println!("ok"),
  //   Err(e) => println!("Error: {}", e)
  // }

  let bootstrap = Version::new(
    "1.19.4",
    VersionType::Release,
    "Umatriz",
    "null",
    "null",
    "E:\\programming\\code\\nomi\\minecraft",
    "E:\\programming\\apps\\java\\bin\\java.exe"
  );

  bootstrap.launch().unwrap();

  tauri::Builder::default()
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}