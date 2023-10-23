use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::repository::launcher_manifest::LauncherManifest;
use crate::configs::consts::LAUNCHER_MANIFEST;

pub mod state;
pub mod download_util;


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
