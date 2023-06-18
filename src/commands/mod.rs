use std::{env::current_dir};

use crate::{downloads::{Download, launcher_manifest::{LauncherManifest, LauncherManifestVersion}}, bootstrap::{Version}};

// FIXME: all this
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

  return Ok(resp.versions);
}