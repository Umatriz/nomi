use std::path::PathBuf;

use crate::downloads::{
    download_file,
    traits::{DownloadResult, DownloadStatus, Downloadable},
};

#[derive(Debug)]
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

    #[tracing::instrument(name = "File download", err, res(level = Level::Trace))]
    #[allow(clippy::blocks_in_conditions)]
    async fn download(self: Box<Self>) -> Self::Out {
        download_file(&self.path, &self.url)
            .await
            .map(|()| DownloadStatus::Success)
    }
}
