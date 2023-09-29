use std::path::Path;

use async_trait::async_trait;

#[async_trait(?Send)]
pub trait DownloadVersion {
    async fn download(&self, dir: impl AsRef<Path>) -> anyhow::Result<()>;

    async fn download_libraries(&self, dir: impl AsRef<Path>) -> anyhow::Result<()>;

    async fn create_json(&self, dir: impl AsRef<Path>) -> anyhow::Result<()>;
}
