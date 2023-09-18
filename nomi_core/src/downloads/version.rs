use std::path::Path;

use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use tokio::{io::AsyncWriteExt, task::JoinSet};

use crate::{
    downloads::assets,
    repository::{launcher_manifest::LauncherManifest, manifest::Manifest},
    version::Version,
};

use super::download_file;

pub struct Vanilla {
    manifest: Manifest,
}

impl Vanilla {
    pub async fn new(version_id: impl Into<String>) -> anyhow::Result<Self> {
        let id = version_id.into();
        let client = Client::new();
        let launcher_manifest = client
            .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
            .send()
            .await?
            .json::<LauncherManifest>()
            .await?;

        let manifest = if let Some(val) = launcher_manifest.versions.iter().find(|i| i.id == id) {
            client
                .get(&val.url)
                .send()
                .await?
                .json::<Manifest>()
                .await?
        } else {
            log::error!("Cannot find this version");

            return Err(crate::error::Error::NoSuchVersion.into());
        };

        Ok(Self { manifest })
    }
}

#[async_trait(?Send)]
impl Version for Vanilla {
    async fn download<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()> {
        let jar_name = format!("{}.jar", &self.manifest.id);
        let versions_path = dir.as_ref().join("versions").join(&self.manifest.id);
        let jar_file = versions_path.join(jar_name);

        download_file(&jar_file, &self.manifest.downloads.client.url.clone()).await?;

        log::info!("Client dowloaded successfully");

        let asset = assets::AssetsDownload::new(
            self.manifest.asset_index.url.clone(),
            self.manifest.asset_index.id.clone(),
        )
        .await?;

        asset.download_assets(dir.as_ref()).await?;
        log::info!("Assets dowloaded successfully");

        asset.get_assets_json(dir.as_ref()).await?;
        log::info!("Assets json created successfully");

        self.create_json(dir.as_ref().join("versions").join(&self.manifest.id))
            .await?;
        self.download_libraries(dir.as_ref()).await?;

        Ok(())
    }

    async fn download_libraries<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()> {
        let mut set = JoinSet::new();

        for lib in self.manifest.libraries.iter() {
            let artifact = lib.downloads.artifact.as_ref();
            if artifact.is_some() {
                let download = artifact.context("`artifact` must be Some")?;
                let path = download
                    .path
                    .clone()
                    .context("`download.path` must be Some")?;
                let final_path = dir.as_ref().join("libraries").join(path);
                set.spawn(download_file(final_path, download.url.clone()));
            }
        }

        while let Some(res) = set.join_next().await {
            res??
        }

        Ok(())
    }

    async fn create_json<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()> {
        let file_name = format!("{}.json", self.manifest.id);
        let path = dir.as_ref().join(file_name);

        let mut file = tokio::fs::File::create(&path).await?;

        let body = serde_json::to_string_pretty(&self.manifest)?;

        file.write_all(body.as_bytes()).await?;

        log::info!(
            "Version json {} created successfully",
            path.to_string_lossy()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn download_test() {
        crate::logger::setup_logger().unwrap();

        let current_dir = std::env::current_dir().unwrap();

        let version = Vanilla::new("1.18.2").await.unwrap();
        version
            .download(current_dir.join("minecraft"))
            .await
            .unwrap();
    }
}
