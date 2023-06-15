#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;
pub mod ui;

use commands::{
  download_version,
  launch,
  get_manifest
};
use ui::Launcher;

use druid::{widget::{Label, Split, Container, Flex, List, LensWrap, TextBox, Button}, Color, Lens, Data, Env};
use druid::{AppLauncher, Widget, WindowDesc};
use druid::im::Vector;
use im::vector;

fn build_ui() -> impl Widget<Launcher> {
  
}

#[tokio::main]
async fn main() {
  let main_window = WindowDesc::new(build_ui())
    .window_size((600.0, 400.0))
    .title("My first Druid App");
  let initial_data = Launcher {
    versions: get_manifest().await.unwrap().into(),
    username: String::new()
  };

  AppLauncher::with_window(main_window)
    .launch(initial_data)
    .expect("Failed to launch application");
}