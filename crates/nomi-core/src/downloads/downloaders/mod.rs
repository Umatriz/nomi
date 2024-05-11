use std::path::PathBuf;

use futures_util::TryFutureExt;
use tokio::sync::mpsc::Sender;

use super::{
    download_file,
    downloadable::{DownloadStatus, Downloadable},
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
    type Out = ();

    async fn download(&self, channel: Sender<DownloadStatus>) -> Self::Out {
        let result = download_file(&self.path, &self.url).await;
        // We can ignore the result since we will guarantee that receiver exists
        let _ = channel.send(DownloadStatus::from(result)).await;
    }
}
