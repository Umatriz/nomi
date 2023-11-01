use std::path::Path;

use anyhow::Context;
use reqwest::Client;
use tokio::{io::AsyncWriteExt, task::JoinSet};

use tracing::{error, info};

use crate::{
    downloads::download_file,
    repository::manifest::{Manifest, ManifestFile},
    utils::get_launcher_manifest,
};

#[derive(Debug)]
pub struct Vanilla {
    manifest: Manifest,
}

impl Vanilla {
    pub async fn new(version_id: impl Into<String>) -> anyhow::Result<Self> {
        let id = version_id.into();
        let client = Client::new();
        let launcher_manifest = get_launcher_manifest().await?;

        let manifest = if let Some(val) = launcher_manifest.versions.iter().find(|i| i.id == id) {
            client
                .get(&val.url)
                .send()
                .await?
                .json::<Manifest>()
                .await?
        } else {
            error!("Cannot find this version");

            return Err(crate::error::Error::NoSuchVersion.into());
        };

        Ok(Self { manifest })
    }

    pub async fn download(
        &self,
        dir: impl AsRef<Path>,
        file_name: impl Into<String>,
    ) -> anyhow::Result<()> {
        let jar_name = format!("{}.jar", file_name.into());
        let path = dir.as_ref().join(jar_name);

        download_file(&path, &self.manifest.downloads.client.url).await?;

        info!("Client downloaded successfully");

        Ok(())
    }

    pub async fn download_libraries(&self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut set = JoinSet::new();

        let mut download_lib = |file: Option<&ManifestFile>| -> anyhow::Result<()> {
            if let Some(download) = file {
                let path = download
                    .path
                    .clone()
                    .context("`download.path` must be Some")?;
                let final_path = dir.as_ref().join(path);
                set.spawn(download_file(final_path, download.url.clone()));
            }

            Ok(())
        };

        for lib in self.manifest.libraries.iter() {
            download_lib(lib.downloads.artifact.as_ref())?;

            if let Some(natives) = lib.downloads.classifiers.as_ref() {
                let native_option = match std::env::consts::OS {
                    "linux" => natives.natives_linux.as_ref(),
                    "windows" => natives.natives_windows.as_ref(),
                    "macos" => natives.natives_macos.as_ref(),
                    _ => unreachable!(),
                };

                download_lib(native_option)?;
            }
        }

        while let Some(res) = set.join_next().await {
            let result = res.unwrap();
            if result.is_err() {
                let str = "MISSING LIBRARY";
                error!("{}", str)
            }
        }

        Ok(())
    }

    pub async fn create_json(&self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let file_name = format!("{}.json", self.manifest.id);
        let path = dir.as_ref().join(file_name);

        if let Some(p) = dir.as_ref().parent() {
            tokio::fs::create_dir_all(p).await?;
        }

        let mut file = tokio::fs::File::create(&path).await?;

        let body = serde_json::to_string_pretty(&self.manifest)?;

        file.write_all(body.as_bytes()).await?;

        info!(
            "Version json {} created successfully",
            path.to_string_lossy()
        );

        Ok(())
    }
}
