use std::path::PathBuf;

pub struct GetPath;

impl GetPath {
  pub fn config() -> PathBuf {
    // TODO: Remove this .join()
    std::env::current_dir().unwrap().join("config.json")
  }

  pub fn game() -> PathBuf {
    std::env::current_dir().unwrap().join("minecraft")
  }
}