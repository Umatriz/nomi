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
    downloads::Download,
    loaders::{fabric::FabricLoader, quilt::QuiltLoader, Loader},
    utils::{logging::setup_logger, GetPath},
};

slint::include_modules!();
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger()?;

    info!("Start");

    let quilt = QuiltLoader::new("1.18.2", None).await.unwrap();

    quilt.download().await.unwrap();

    // let fabric = FabricLoader::new("1.18.2").await.unwrap();

    // fabric.download().await.unwrap();

    let ui = MainWindow::new().unwrap();
    ui.global::<State>().on_launch(|_id| {
        tokio::spawn(download_version("id".to_string()));
    });
    ui.run().unwrap();

    Ok(())
}
