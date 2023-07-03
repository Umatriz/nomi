use crate::{downloads::{Download, launcher_manifest::{LauncherManifest, LauncherManifestVersion}}, utils::GetPath, configs::launcher::Launcher, bootstrap::{ClientSettings, ClientBootstrap, ClientAuth, ClientVersion}};

use serde::Serialize;
use tauri::Window;

#[derive(Serialize, Clone)]
struct Downloading {
  state: bool,
}

#[tauri::command]
pub async fn download_version(id: String, window: Window) -> Result<(), ()> {
  let load: Download = Download::new().await;

  window.emit("downloading", Downloading {
    state: true
  }).unwrap();

  // load.download(id, GetPath::game().to_str().unwrap().to_string())
  //   .await
  //   .unwrap();

  tokio::time::sleep(std::time::Duration::from_millis(5000)).await;

  window.emit("downloading", Downloading {
    state: false
  }).unwrap();

  Ok(())
}

#[tauri::command]
pub async fn get_manifest() -> Result<Vec<LauncherManifestVersion>, ()> {
  let resp: LauncherManifest = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
    .await
    .unwrap()
    .json()
    .await
    .unwrap();

  return Ok(resp.versions);
}

#[tauri::command]
pub async fn get_config() -> Result<Launcher, ()> {
  let launcher_config = Launcher::from_file(None);

  Ok(launcher_config)
}


#[tauri::command]
pub async fn launch(username: String, version: String) -> Result<(), ()> {
  let bootstrap = ClientBootstrap::new(ClientSettings {
    assets: GetPath::game().join("assets"),
    auth: ClientAuth {
      username: username,
      access_token: None,
      uuid: Some(uuid::Uuid::new_v4().to_string()),
    },
    game_dir: GetPath::game(),
    java_bin: GetPath::java_bin().unwrap(),
    libraries_dir: GetPath::game().join("libraries"),
    manifest_file: GetPath::game()
      .join("versions")
      .join(&version)
      .join(format!("{}.json", version)),
    natives_dir: GetPath::game().join("versions").join(&version).join("natives"),
    version: ClientVersion {
      version: version.clone(),
      version_type: "release".to_string(),
    },
    version_jar_file: GetPath::game()
      .join("versions")
      .join(&version)
      .join(format!("{}.jar", version)),
  });
  
  bootstrap.launch().unwrap();
  
  Ok(())
}

