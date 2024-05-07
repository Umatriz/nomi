use std::path::Path;

#[async_trait::async_trait]
pub trait DownloadVersion {
    async fn download(&self, dir: &Path, file_name: &str) -> anyhow::Result<()>;
    async fn download_libraries(&self, dir: &Path) -> anyhow::Result<()>;
    async fn create_json(&self, dir: &Path) -> anyhow::Result<()>;
}

const _: Option<Box<dyn DownloadVersion>> = None;
