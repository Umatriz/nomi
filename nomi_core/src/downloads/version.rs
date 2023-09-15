use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use reqwest::Client;
use thiserror::Error;

use super::{assets, download_file};
use crate::repository::launcher_manifest::{LauncherManifest, LauncherManifestVersion};
use crate::repository::manifest::Manifest;

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
        Ok(Self {
            global_manifest: Self::init().await?,
        })
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

    pub async fn create_version_json(
        &self,
        manifest: &Manifest,
        version_dir: PathBuf,
    ) -> Result<()> {
        let filen = format!("{}.json", manifest.id);
        let path = version_dir.join(filen);

        let file = std::fs::File::create(&path)?;

        let _ = serde_json::to_writer_pretty(&file, &manifest);

        log::info!(
            "Version json {} created successfully",
            path.to_string_lossy()
        );

        Ok(())
    }

    async fn download_version(&self, manifest: Manifest, dir: &Path) -> Result<()> {
        let jar_name = format!("{}.jar", &manifest.id);
        let versions_path = dir.join("versions").join(&manifest.id);
        let jar_file = versions_path.join(jar_name);

        download_file(&jar_file, manifest.downloads.client.url.clone()).await?;

        log::info!("Client dowloaded successfully");

        let asset = assets::AssetsDownload::new(
            manifest.asset_index.url.clone(),
            manifest.asset_index.id.clone(),
        )
        .await?;

        asset.download_assets(dir).await?;
        log::info!("Assets dowloaded successfully");

        asset.get_assets_json(dir).await?;
        log::info!("Assets json created successfully");

        self.create_version_json(&manifest, versions_path).await?;

        for lib in manifest.libraries {
            let artifact = lib.downloads.artifact;
            if artifact.is_some() {
                let download = artifact.context("`artifact` must be Some")?;
                let path = download.path.context("`download.path` must be Some")?;
                let final_path = dir.join("libraries").join(path);
                download_file(&final_path, download.url).await?;
            }
        }

        Ok(())
    }

    pub async fn download(&self, version_id: String, dir: &PathBuf) -> Result<()> {
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
