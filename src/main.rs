pub mod bootstrap;
pub mod commands;
pub mod configs;
pub mod downloads;
pub mod loaders;
pub mod manifest;
pub mod utils;

use commands::download_version;

use chrono::offset::Local;
use log::info;
use std::{fs::File, path::PathBuf};

use crate::{downloads::Download, utils::GetPath};

slint::include_modules!();
#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    info!("Start");

    // tokio::spawn(async {
    //     let version = Download::new().await;

    //     version
    //         .download(
    //             "1.18.2".to_string(),
    //             GetPath::game().to_string_lossy().to_string(),
    //         )
    //         .await
    //         .unwrap();
    // });

    let ui = MainWindow::new().unwrap();
    ui.global::<State>().on_launch(|_id| {
        tokio::spawn(download_version("id".to_string()));
    });
    ui.run().unwrap();
}
