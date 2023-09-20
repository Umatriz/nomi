use std::path::Path;

use async_trait::async_trait;

#[async_trait(?Send)]
pub trait Version {
    async fn download<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()>;

    async fn download_libraries<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()>;

    async fn create_json<P: AsRef<Path>>(&self, dir: P) -> anyhow::Result<()>;
}
