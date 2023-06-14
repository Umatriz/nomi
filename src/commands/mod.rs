use std::{env::current_dir};

use crate::{downloads::{Download, launcher_manifest::{LauncherManifest, LauncherManifestVersion}}, bootstrap::{Version}};
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

pub async fn get_manifest() -> Result<Vec<LauncherManifestVersion>, ()> {
  let resp: LauncherManifest = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
    .await
    .unwrap()
    .json()
    .await
    .unwrap();

  // let vec: Vec<LauncherManifestVersion> = vec![
  //   LauncherManifestVersion {
  //     id: "1".to_string(),
  //     version_type: "release".to_string(),
  //     url: "test".to_string(),
  //     time: "12:00".to_string(),
  //     release_time: "12:00".to_string()
  //   },
  //   LauncherManifestVersion {
  //     id: "2".to_string(),
  //     version_type: "release".to_string(),
  //     url: "test".to_string(),
  //     time: "12:00".to_string(),
  //     release_time: "12:00".to_string()
  //   },
  //   LauncherManifestVersion {
  //     id: "3".to_string(),
  //     version_type: "release".to_string(),
  //     url: "test".to_string(),
  //     time: "12:00".to_string(),
  //     release_time: "12:00".to_string()
  //   }
  // ];

  return Ok(resp.versions);
}