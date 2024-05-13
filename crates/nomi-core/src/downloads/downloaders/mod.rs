use std::path::PathBuf;

use super::{
    download_file,
    downloadable::{DownloadResult, DownloadStatus, Downloadable},
};

pub struct AssetDownloader {
    url: String,
    path: PathBuf,
}

impl AssetDownloader {
    pub fn new(url: String, path: PathBuf) -> Self {
        Self { url, path }
    }
}

#[async_trait::async_trait]
impl Downloadable for AssetDownloader {
    type Out = DownloadResult;

    async fn download(&self) -> Self::Out {
        download_file(&self.path, &self.url)
            .await
            .map(|_| DownloadStatus::Success)
    }
}
