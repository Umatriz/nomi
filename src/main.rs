pub mod bootstrap;
pub mod commands;
pub mod configs;
pub mod downloads;
pub mod loaders;
pub mod manifest;
pub mod utils;

use commands::download_version;

extern crate log;
extern crate simplelog;

use simplelog::*;

use chrono::offset::Local;
use std::{fs::File, path::PathBuf};

slint::include_modules!();
#[tokio::main]
async fn main() {
    let time = Local::now().time().to_string().replace(':', ".");
    let log_time = time.split('.').collect::<Vec<_>>();

    let path = PathBuf::new().join("nomi_logs").join(format!(
        "{}_{}.{}.{}.log",
        Local::now().date_naive(),
        log_time[0],
        log_time[1],
        log_time[2]
    ));

    let _ = std::fs::create_dir_all(path.parent().unwrap());

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(path).unwrap(),
        ),
    ])
    .unwrap();

    info!("Start");

    let ui = MainWindow::new().unwrap();
    ui.global::<State>().on_launch(|_id| {
        tokio::spawn(download_version("id".to_string()));
    });
    ui.run().unwrap();
}
