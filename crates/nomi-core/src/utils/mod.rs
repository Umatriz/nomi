use std::path::Path;

use reqwest::Client;
use serde::de::DeserializeOwned;
use tokio::io::AsyncWriteExt;

use crate::repository::launcher_manifest::LauncherManifest;

pub mod maven;
pub mod state;

pub const LAUNCHER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

pub async fn get<T: DeserializeOwned>(url: impl Into<String>) -> anyhow::Result<T> {
    Ok(reqwest::get(url.into()).await?.json::<T>().await?)
}

pub async fn get_launcher_manifest() -> anyhow::Result<LauncherManifest> {
    tracing::debug!("Calling Launcher Manifest");
    Ok(Client::new()
        .get(LAUNCHER_MANIFEST)
        .send()
        .await?
        .json::<LauncherManifest>()
        .await?)
}

pub async fn write_into_file(data: &[u8], path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    let mut file = tokio::fs::File::create(&path).await?;

    file.write_all(data).await?;

    Ok(())
}

pub fn path_to_string(p: impl AsRef<Path>) -> String {
    p.as_ref().to_string_lossy().to_string()
}
