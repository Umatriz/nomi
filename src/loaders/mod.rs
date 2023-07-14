use async_trait::async_trait;
use log::trace;

pub mod fabric;
pub mod fabric_meta;

pub mod maven;

pub const QUILT_META: &str = "https://meta.quiltmc.org/";
pub const QUILT_MAVEN: &str = "https://maven.quiltmc.org/";

pub const FABRIC_META: &str = "https://meta.fabricmc.net/v2";
pub const FABRIC_MAVEN: &str = "https://maven.fabricmc.net";

#[async_trait(?Send)]
pub trait Loader {
    async fn download(&self) -> anyhow::Result<()>;

    fn create_json(&self) -> anyhow::Result<()>;

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
