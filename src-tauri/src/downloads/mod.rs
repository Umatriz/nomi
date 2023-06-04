use std::{
  path::{Path, PathBuf}
};

use reqwest::{Client, blocking};
use thiserror::Error;
use tokio::task::spawn_blocking;

mod launcher_manifest;
pub mod assets;

use crate::manifest::Manifest;
use launcher_manifest::{LauncherManifest, LauncherManifestVersion};


#[derive(Error, Debug)]
pub enum DownloaderError {
  #[error("An unexpected error has ocurred.")]
  UnknownError,

  #[error("No such version")]
  NoSuchVersion,

  #[error("{0}")]
  Request(#[from] reqwest::Error),

  #[error("{0}")]
  Json(#[from] serde_json::Error),
}

pub struct Download {
  global_manifest: LauncherManifest
}

impl Download {
  pub async fn new() -> Self {
    Self {
      global_manifest: Self::init()
        .await
        .unwrap(),
    }
  }

  async fn init() -> Result<LauncherManifest, DownloaderError> {
    let data: LauncherManifest = Client::new()
      .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
      .send()
      .await?
      .json()
      .await?;
    
    return Ok(data);
  }

  fn get_version(&self, id: String) -> Option<&LauncherManifestVersion> {
    for version in &self.global_manifest.versions {
      if version.id == id {
          return Some(&version);
      }
    }
    return None;
  }

  async fn dowload_file<P: AsRef<Path>>(&self, path: P, url: String) {
    let path = path.as_ref();
    let _ = std::fs::create_dir_all(path.parent().unwrap());

    let mut file = std::fs::File::create(path).unwrap();

    let _response = spawn_blocking(move || {
      blocking::get(url)
        .unwrap()
        .copy_to(&mut file)
        .unwrap()
    }).await;
    println!("Downloaded {}", path.to_str().unwrap().to_string())
  }

  pub async fn create_version_json(&self, manifest: &Manifest, version_dir: PathBuf) -> Result<(), serde_json::Error> {
    let filen = format!("{}.json", manifest.id);
    let path = version_dir.join(filen);

    let file = std::fs::File::create(&path).unwrap();

    let _ = serde_json::to_writer_pretty(&file, &manifest);

    Ok(())
  }
  
  async fn download_version(&self, manifest: Manifest, dir: String) -> Result<(), DownloaderError> {
    let main_dir = Path::new(&dir);
    let jar_name = format!("{}.jar", &manifest.id);
    let versions_path = main_dir
      .join("versions")
      .join(&manifest.id);
    let jar_file = versions_path
      .join(jar_name);

    self.dowload_file(&jar_file, manifest.downloads.client.url.clone()).await;

    let asset = assets::AssetsDownload::new(manifest.asset_index.url.clone(), manifest.asset_index.id.clone()).await;
    asset.download_assets(&dir).await;
    asset.get_assets_json(&dir).await?;

    self.create_version_json(&manifest, versions_path).await?;

    for lib in manifest.libraries {
      let artifact = lib.downloads.artifact;
      if artifact.is_some() {
        let download = artifact.unwrap();
        let path = download.path.unwrap();
        let final_path = main_dir
          .join("libraries")
          .join(path)
          .to_str()
          .unwrap()
          .to_string()
          .replace("/", "\\");
        self.dowload_file(&final_path, download.url).await;
      }
    }

    Ok(())
  }


  pub async fn download(&self, version_id: String, dir: String,) -> Result<(), DownloaderError> {
    let client = Client::new();
    let version_option = self.get_version(version_id);

    if version_option.is_none() {
      return Err(DownloaderError::NoSuchVersion);
    }

    let version = version_option.unwrap();
    let data = client
      .get(&version.url)
      .send()
      .await?
      .json()
      .await?;
    
    self.download_version(data, dir).await?;

    Ok(())
  }
}