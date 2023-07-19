pub mod bootstrap;
pub mod commands;
pub mod configs;
pub mod downloads;
pub mod loaders;

pub mod manifest;
pub mod utils;

use commands::download_version;

use log::info;

use std::time::SystemTime;

use crate::{
    commands::launch,
    downloads::Download,
    loaders::{fabric::FabricLoader, quilt::QuiltLoader, Loader},
    utils::{logging::setup_logger, GetPath},
};

slint::include_modules!();
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger()?;

    info!("Start");

    Download::new()
        .await
        .download(
            "1.18.2".to_string(),
            GetPath::game().to_string_lossy().to_string(),
        )
        .await
        .unwrap();

    // launch("username".to_string(), "1.18.2".to_string())
    //     .await
    //     .unwrap();

    let ui = MainWindow::new().unwrap();
    ui.global::<State>().on_launch(|_id| {
        tokio::spawn(download_version("id".to_string()));
    });
    ui.run().unwrap();

    Ok(())
}
