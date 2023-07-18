pub mod bootstrap;
pub mod commands;
pub mod configs;
pub mod downloads;
pub mod loaders;
pub mod profiles;
pub mod manifest;
pub mod utils;

use commands::download_version;

slint::include_modules!();
#[tokio::main]
async fn main() {
    tokio::spawn(async {});
    let ui = MainWindow::new().unwrap();
    ui.global::<State>().on_launch(|_id| {});
    ui.run().unwrap();
}
