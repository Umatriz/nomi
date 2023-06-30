use crate::{downloads::{Download, launcher_manifest::{LauncherManifest, LauncherManifestVersion}}, utils::GetPath, configs::launcher::Launcher};

// FIXME: Change all `String` in paths to `PathBuf`
#[tauri::command]
pub async fn download_version(id: String) -> Result<(), ()> {
  let load: Download = Download::new().await;
  load.download(id, GetPath::game().to_str().unwrap().to_string())
    .await
    .unwrap();

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

