pub mod utils;
pub mod downloads;
pub mod bootstrap;
pub mod manifest;
pub mod commands;

use commands::{
  download_version,
  launch,
  get_manifest
};

#[tokio::main]
async fn main() {
  
}