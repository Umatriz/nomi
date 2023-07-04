#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;
pub mod configs;
pub mod loaders;

use commands::{download_version, get_manifest, get_config, launch};

#[tokio::main]
async fn main() {
  
}