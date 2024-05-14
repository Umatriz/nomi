use std::path::PathBuf;

use crate::downloads::{
    download_file,
    downloadable::{DownloadResult, DownloadStatus, Downloadable},
};

pub struct FileDownloader {
    url: String,
    path: PathBuf,
}

impl FileDownloader {
    pub fn new(url: String, path: PathBuf) -> Self {
        Self { url, path }
    }
}

#[async_trait::async_trait]
impl Downloadable for FileDownloader {
    type Out = DownloadResult;

    async fn download(&self) -> Self::Out {
        download_file(&self.path, &self.url)
            .await
            .map(|_| DownloadStatus::Success)
    }
}
