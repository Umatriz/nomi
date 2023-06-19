use crate::{downloads::{Download, launcher_manifest::{LauncherManifest, LauncherManifestVersion}}, bootstrap::{Version}, utils::GetPath};

pub struct Commands;

// FIXME: Change all `String` in paths to `PathBuf`
impl Commands {
  pub async fn download_version(id: &str) {
    let load: Download = Download::new().await;
    load.download(id.to_string(), GetPath::game().to_str().unwrap().to_string())
      .await
      .unwrap();
  }
  
  pub fn launch (
    id: &str,
    version_type: &String,
    username: &str,
    java_bin: &str
  ) {
    let bootstrap = Version::new(
      id,
      version_type,
      username,
      "null",
      GetPath::game().to_str().unwrap(),
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
}
