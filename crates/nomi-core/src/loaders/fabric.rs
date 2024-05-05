use std::path::Path;

use reqwest::Client;
use tokio::task::JoinSet;
use tracing::info;

use crate::{
    downloads::{download_file, download_version::DownloadVersion},
    repository::{fabric_meta::FabricVersions, fabric_profile::FabricProfile},
    utils::{
        maven::MavenData,
        state::{launcher_manifest_state_try_init, LAUNCHER_MANIFEST_STATE},
        write_into_file,
    },
};

use super::vanilla::Vanilla;

#[derive(Debug)]
pub struct Fabric {
    pub game_version: String,
    pub profile: FabricProfile,
}

impl Fabric {
    pub async fn new(
        game_version: impl Into<String>,
        loader_version: Option<impl Into<String>>,
    ) -> anyhow::Result<Self> {
        let game_version = game_version.into();

        let client = Client::new();
        let launcher_manifest = LAUNCHER_MANIFEST_STATE
            .get_or_try_init(launcher_manifest_state_try_init)
            .await?;

        if !launcher_manifest
            .launcher
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

        let profile_version = loader_version
            .map(|s| s.into())
            .and_then(|loader| versions.iter().find(|i| i.loader.version == loader))
            .unwrap_or_else(|| &versions[0]);

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
            profile,
            game_version,
        })
    }
}

#[async_trait::async_trait]
impl DownloadVersion for Fabric {
    async fn download(
        &self,
        dir: impl AsRef<Path> + Send,
        file_name: impl Into<String> + Send,
    ) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        Vanilla::new(&self.game_version)
            .await?
            .download(dir, file_name)
            .await?;

        info!("Fabric downloaded successfully");

        Ok(())
    }

    async fn download_libraries(&self, dir: impl AsRef<Path> + Send + Sync) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        let mut set = JoinSet::new();

        self.profile.libraries.iter().for_each(|lib| {
            let maven = MavenData::new(&lib.name);
            let path = dir.join(maven.path);
            if !path.exists() {
                set.spawn(download_file(path, format!("{}{}", lib.url, maven.url)));
            }
        });

        while let Some(res) = set.join_next().await {
            res??
        }

        Ok(())
    }

    async fn create_json(&self, dir: impl AsRef<Path> + Send) -> anyhow::Result<()> {
        let file_name = format!("{}.json", self.profile.id);
        let path = dir.as_ref().join(file_name);

        let body = serde_json::to_string_pretty(&self.profile)?;

        write_into_file(body.as_bytes(), &path).await?;

        info!(
            "Version json {} created successfully",
            path.to_string_lossy()
        );

        Ok(())
    }
}

impl From<FabricProfile> for Fabric {
    fn from(value: FabricProfile) -> Self {
        Self {
            game_version: value.inherits_from.clone(),
            profile: value,
        }
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
