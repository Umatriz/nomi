use std::{env::current_dir, path::PathBuf};

pub fn get_main_dir() -> PathBuf {
  let dir = current_dir().unwrap();
  return dir;
}