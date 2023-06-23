use crate::{downloads::{Download, launcher_manifest::{LauncherManifest, LauncherManifestVersion}}, utils::GetPath};

pub struct Commands;

// FIXME: Change all `String` in paths to `PathBuf`
impl Commands {
  pub async fn download_version(id: String) {
    let load: Download = Download::new().await;
    load.download(id, GetPath::game().to_str().unwrap().to_string())
      .await
      .unwrap();
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
}
