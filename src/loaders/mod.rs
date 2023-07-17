use std::path::PathBuf;

use async_trait::async_trait;
use log::{info, trace};
use reqwest::Client;

use self::errors::LoaderError;
use crate::{downloads::Download, manifest::Manifest};

pub mod fabric;
pub mod quilt;

pub mod errors;
pub mod maven;

pub const QUILT_META: &str = "https://meta.quiltmc.org/";
pub const QUILT_MAVEN: &str = "https://maven.quiltmc.org/";

pub const FABRIC_META: &str = "https://meta.fabricmc.net/v2";
pub const FABRIC_MAVEN: &str = "https://maven.fabricmc.net";

#[async_trait(?Send)]
pub trait Loader {
    async fn download(&self) -> anyhow::Result<()>;

    fn create_json(&self) -> anyhow::Result<()>;

    async fn get_local_manifest(&self, version: &str, version_dir: PathBuf) -> anyhow::Result<()> {
        let launcher_manifest = Download::init().await?;

        let url = match launcher_manifest.versions.iter().find(|x| x.id == version) {
            Some(manifest) => &manifest.url,
            // i think this error will never call
            None => return Err(LoaderError::LauncherManifestVersionError.into()),
        };

        let manifest: Manifest = Client::new().get(url).send().await?.json().await?;

        let filen = format!("{}.json", manifest.id);
        let path = version_dir.join(filen);

        let file = std::fs::File::create(&path).unwrap();

        let _ = serde_json::to_writer_pretty(&file, &manifest);

        info!(
            "Version json {} created successfully",
            path.to_string_lossy()
        );

        Ok(())
    }

    async fn dowload_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        url: String,
    ) -> anyhow::Result<()> {
        let path = path.as_ref();
        let _ = std::fs::create_dir_all(path.parent().unwrap());

        let mut file = std::fs::File::create(path)?;

        let log_url = url.clone();
        let _response = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            reqwest::blocking::get(url)?.copy_to(&mut file)?;
            Ok(())
        })
        .await??;

        trace!(
            "Dowloaded successfully. url: {}, path: {}",
            log_url,
            path.to_string_lossy()
        );

        Ok(())
    }
}
