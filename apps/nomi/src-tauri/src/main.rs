// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use std::path::Path;

use downloads::jvm_dowload::*;

pub fn setup_logger() -> Result<(), fern::InitError> {
    let logs = std::env::current_dir()?.join("logs");

    let _ = std::fs::create_dir_all(logs);

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::DateBased::new("logs/", "%Y-%m-%d-nomi.log"))
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    setup_logger().unwrap();

    download_java(
        &std::env::current_dir().unwrap(),
        &std::env::current_dir().unwrap().join("java"),
    )
    .await
    .unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
