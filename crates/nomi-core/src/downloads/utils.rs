use reqwest::Client;

use crate::repository::launcher_manifest::LauncherManifest;

use super::LAUNCHER_MANIFEST;

pub async fn get_launcher_manifest() -> anyhow::Result<LauncherManifest> {
    Ok(Client::new()
        .get(LAUNCHER_MANIFEST)
        .send()
        .await?
        .json::<LauncherManifest>()
        .await?)
}
