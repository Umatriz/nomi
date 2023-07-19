pub mod bootstrap;
pub mod commands;
pub mod configs;
pub mod downloads;
pub mod loaders;

pub mod manifest;
pub mod utils;

use commands::download_version;

use log::info;

use crate::utils::logging::setup_logger;

slint::include_modules!();
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logger()?;

    info!("Start");

    let ui = MainWindow::new().unwrap();
    ui.global::<State>().on_launch(|_id| {
        tokio::spawn(download_version("id".to_string()));
    });
    ui.run().unwrap();

    Ok(())
}
