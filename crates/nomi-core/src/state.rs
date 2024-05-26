use anyhow::Context;
use reqwest::{get, Client};
use tokio::sync::OnceCell;

use crate::repository::{
    launcher_manifest::{LauncherManifest, Version},
    manifest::Manifest,
};

// TODO: Write helper functions for quick access

pub const LAUNCHER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
pub static LAUNCHER_MANIFEST_STATE: OnceCell<LauncherManifest> = OnceCell::const_new();

pub async fn get_launcher_manifest_owned() -> anyhow::Result<LauncherManifest> {
    tracing::debug!("Calling Launcher Manifest");
    Ok(Client::new()
        .get(LAUNCHER_MANIFEST)
        .send()
        .await?
        .json::<LauncherManifest>()
        .await?)
}

pub async fn get_launcher_manifest() -> anyhow::Result<&'static LauncherManifest> {
    LAUNCHER_MANIFEST_STATE
        .get_or_try_init(get_launcher_manifest_owned)
        .await
}

impl LauncherManifest {
    pub fn find_version(&self, version: impl Into<String>) -> Option<&Version> {
        let version = version.into();
        self.versions.iter().find(|v| v.id == version)
    }

    pub async fn get_version_manifest(
        &self,
        version: impl Into<String>,
    ) -> anyhow::Result<Manifest> {
        let url = &self
            .find_version(version)
            .context("cannot find such version")?
            .url;

        get(url).await?.json().await.map_err(Into::into)
    }

    pub async fn get_version_manifest_content(
        &self,
        version: impl Into<String>,
    ) -> anyhow::Result<String> {
        let url = &self
            .find_version(version)
            .context("cannot find such version")?
            .url;

        Ok(Client::new().get(url).send().await?.text().await?)
    }
}
