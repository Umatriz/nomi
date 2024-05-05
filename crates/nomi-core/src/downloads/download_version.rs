use std::path::Path;

#[async_trait::async_trait]
pub trait DownloadVersion {
    async fn download(
        &self,
        dir: impl AsRef<Path> + Send,
        file_name: impl Into<String> + Send,
    ) -> anyhow::Result<()>;
    async fn download_libraries(&self, dir: impl AsRef<Path> + Send + Sync) -> anyhow::Result<()>;
    async fn create_json(&self, dir: impl AsRef<Path> + Send) -> anyhow::Result<()>;
}
