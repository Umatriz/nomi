use std::path::Path;

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    downloads::{utils::get_launcher_manifest, version::DownloadVersion},
    repository::{fabric_meta::FabricVersions, fabric_profile::FabricProfile},
};

pub struct Fabric {
    versions: FabricVersions,
    profile: FabricProfile,
}

impl Fabric {
    pub async fn new(
        game_version: impl Into<String>,
        loader_version: Option<impl Into<String>>,
    ) -> anyhow::Result<Self> {
        let game_version = game_version.into();

        let client = Client::new();
        let launcher_manifest = get_launcher_manifest().await?;
        if !launcher_manifest
            .versions
            .iter()
            .any(|v| v.id == game_version)
        {
            return Err(crate::error::Error::NoSuchVersion.into());
        };

        let versions: FabricVersions = client
            .get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}",
                game_version
            ))
            .send()
            .await?
            .json()
            .await?;

        let profile_version = if let Some(loader) = loader_version {
            let loader = loader.into();
            if let Some(loader) = versions.iter().find(|i| i.loader.version == loader) {
                loader
            } else {
                &versions[0]
            }
        } else {
            &versions[0]
        };

        let profile: FabricProfile = client
            .get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                game_version, profile_version.loader.version
            ))
            .send()
            .await?
            .json()
            .await?;

        Ok(Self { versions, profile })
    }
}

#[async_trait(?Send)]
impl DownloadVersion for Fabric {
    async fn download(&self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn download_libraries(&self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn create_json(&self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        Ok(())
    }
}
