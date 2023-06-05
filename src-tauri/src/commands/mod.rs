use std::env::current_dir;

use crate::{downloads::Download, bootstrap::{Version}};

#[tauri::command]
pub async fn download_version(id: &str) -> Result<(), ()> {
  let load = Download::new().await;
  let minecraft_dir = current_dir().unwrap()
    .join("minecraft")
    .to_str()
    .unwrap()
    .to_string();
  load.download(id.to_string(), minecraft_dir)
    .await
    .unwrap();

  Ok(())
}

#[tauri::command]
pub fn launch (
  id: &str,
  version_type: String,
  username: &str,
  java_bin: &str
) {
  let bootstrap = Version::new(
    id,
    version_type,
    username,
    "null",
    "null",
    // current_dir()
    //   .unwrap()
    //   .join("minecraft")
    //   .to_str()
    //   .unwrap(),
    "E:\\programming\\code\\nomi\\minecraft",
    java_bin
  );

  bootstrap.launch().unwrap();
}