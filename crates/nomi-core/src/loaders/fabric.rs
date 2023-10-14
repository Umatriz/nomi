use std::path::Path;

use async_trait::async_trait;
use reqwest::Client;
use tokio::{io::AsyncWriteExt, task::JoinSet};
use tracing::info;

use crate::{
    downloads::{download_file, utils::get_launcher_manifest},
    repository::{fabric_meta::FabricVersions, fabric_profile::FabricProfile},
    version::download::DownloadVersion,
};

use super::{maven::MavenData, vanilla::Vanilla};

#[derive(Debug)]
pub struct Fabric {
    game_version: String,
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

        Ok(Self {
            versions,
            profile,
            game_version,
        })
    }
}

#[async_trait(?Send)]
impl DownloadVersion for Fabric {
    async fn download(
        &self,
        dir: impl AsRef<Path>,
        file_name: impl Into<String>,
    ) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        Vanilla::new(&self.game_version)
            .await?
            .download(dir, file_name)
            .await?;

        info!("Fabric downloaded successfully");

        Ok(())
    }

    async fn download_libraries(&self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        let mut set = JoinSet::new();

        self.profile.libraries.iter().for_each(|lib| {
            let maven = MavenData::new(&lib.name);
            set.spawn(download_file(
                dir.join(maven.path),
                format!("{}{}", lib.url, maven.url),
            ));
        });

        while let Some(res) = set.join_next().await {
            res??
        }

        Ok(())
    }

    async fn create_json(&self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let file_name = format!("{}.json", self.profile.id);
        let path = dir.as_ref().join(file_name);

        if let Some(p) = dir.as_ref().parent() {
            tokio::fs::create_dir_all(p).await?;
        }

        let mut file = tokio::fs::File::create(&path).await?;

        let body = serde_json::to_string_pretty(&self.profile)?;

        file.write_all(body.as_bytes()).await?;

        info!(
            "Version json {} created successfully",
            path.to_string_lossy()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use super::*;

    #[tokio::test]
    async fn download_test() {
        let subscriber = tracing_subscriber::fmt().pretty().finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();

        let cur = current_dir().unwrap();

        Fabric::new("1.18.2", None::<String>)
            .await
            .unwrap()
            .download(cur.join("minecraft"), "1.18.2.fabric")
            .await
            .unwrap();
    }
}
