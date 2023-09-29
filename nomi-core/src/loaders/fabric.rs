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
        loader_version: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let game_version = game_version.into();
        let loader_version = loader_version.into();

        let client = Client::new();
        let launcher_manifest = get_launcher_manifest().await?;
        if !launcher_manifest
            .versions
            .iter()
            .any(|v| v.id == game_version)
        {
            return Err(crate::error::Error::NoSuchVersion.into());
        };
        let versions = client.get("");

        Ok(Self {
            versions: todo!(),
            profile: todo!(),
        })
    }
}

#[async_trait(?Send)]
impl DownloadVersion for Fabric {
    async fn download<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()> {
        Ok(())
    }

    async fn download_libraries<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()> {
        Ok(())
    }

    async fn create_json<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()> {
        Ok(())
    }
}
