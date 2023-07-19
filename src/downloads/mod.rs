use std::path::{Path, PathBuf};

use log::{info, trace};
use anyhow::{Result, Context};
use reqwest::{blocking, Client};
use thiserror::Error;
use tokio::task::spawn_blocking;

pub mod assets;
pub mod java_installer;
pub mod launcher_manifest;

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
    global_manifest: LauncherManifest,
}

impl Download {
    pub async fn new() -> Result<Self> {
        Ok(
            Self {
                global_manifest: Self::init().await?,
            }
        )
    }

    pub async fn init() -> Result<LauncherManifest, DownloaderError> {
        let data: LauncherManifest = Client::new()
            .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .send()
            .await?
            .json()
            .await?;

        Ok(data)
    }

    fn get_version(&self, id: String) -> Option<&LauncherManifestVersion> {
        self.global_manifest
            .versions
            .iter()
            .find(|&version| version.id == id)
    }

    async fn dowload_file<P: AsRef<Path>>(&self, path: P, url: String) -> Result<()>{
        let path = path.as_ref();
        let _ = std::fs::create_dir_all(path.parent().context("failed to get parent dir")?);

        let mut file = std::fs::File::create(path).context("failed to create file")?;

        let _response =
            spawn_blocking(move || blocking::get(url).unwrap().copy_to(&mut file).unwrap()).await;

        trace!("Downloaded successfully {}", path.to_string_lossy());
    }

    pub async fn create_version_json(
        &self,
        manifest: &Manifest,
        version_dir: PathBuf,
    ) -> Result<()> {
        let filen = format!("{}.json", manifest.id);
        let path = version_dir.join(filen);

        let file = std::fs::File::create(path)?;

        let _ = serde_json::to_writer_pretty(&file, &manifest);

        info!(
            "Version json {} created successfully",
            path.to_string_lossy()
        );

        Ok(())
    }

    async fn download_version(
        &self,
        manifest: Manifest,
        dir: String,
    ) -> Result<()> {
        let main_dir = Path::new(&dir);
        let jar_name = format!("{}.jar", &manifest.id);
        let versions_path = main_dir.join("versions").join(&manifest.id);
        let jar_file = versions_path.join(jar_name);

        self.dowload_file(&jar_file, manifest.downloads.client.url.clone())
            .await?;

        info!("Client dowloaded successfully");

        let asset = assets::AssetsDownload::new(
            manifest.asset_index.url.clone(),
            manifest.asset_index.id.clone(),
        )
        .await;

        asset.download_assets(&dir).await;
        info!("Assets dowloaded successfully");

        asset.get_assets_json(&dir).await?;
        info!("Assets json created successfully");

        self.create_version_json(&manifest, versions_path).await?;

        for lib in manifest.libraries {
            let artifact = lib.downloads.artifact;
            if artifact.is_some() {
                let download = artifact.context("`artifact` must be Some")?;
                let path = download.path.context("`download.path` must be Some")?;
                let final_path = main_dir
                    .join("libraries")
                    .join(path)
                    .to_str().context("")?
                    .to_string()
                    .replace('/', "\\");
                self.dowload_file(&final_path, download.url).await?;
            }
        }

        Ok(())
    }

    pub async fn download(&self, version_id: String, dir: String) -> Result<()> {
        let client = Client::new();
        let version_option = self.get_version(version_id);

        if version_option.is_none() {
            return Err(DownloaderError::NoSuchVersion.into());
        }

        let version = version_option.context("version_option must be Some")?;
        let data = client.get(&version.url).send().await?.json().await?;

        self.download_version(data, dir).await?;

        Ok(())
    }
}
