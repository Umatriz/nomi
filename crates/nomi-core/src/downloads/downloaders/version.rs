use crate::downloads::downloadable::{DownloadResult, Downloader, DownloaderIO};

pub struct VersionDownloader {
    downloader: Box<dyn Downloader<Data = DownloadResult>>,
    io: Box<dyn DownloaderIO>,
}

#[async_trait::async_trait]
pub trait DownloadableVersion {
    async fn downloader(&self) -> VersionDownloader;
}
