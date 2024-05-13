use tokio::sync::mpsc::Sender;

use super::DownloadError;

pub type DownloadResult = Result<DownloadStatus, DownloadError>;

#[must_use]
#[derive(Debug)]
pub enum DownloadStatus {
    /// Downloaded successfully
    Success,
    /// Downloaded successfully certain amount of elements
    SuccessWithProgress(u32),
}

impl DownloadStatus {
    pub fn progress(progress: u32) -> Self {
        Self::SuccessWithProgress(progress)
    }
}

#[async_trait::async_trait]
pub trait Downloadable: Send + Sync {
    type Out: Send;

    async fn download(&self) -> Self::Out;
}

const _: Option<Box<dyn Downloadable<Out = DownloadResult>>> = None;

#[async_trait::async_trait]
pub trait Downloader: Send + Sync {
    type Data;

    async fn download(&self, channel: Sender<Self::Data>);
}

const _: Option<Box<dyn Downloader<Data = DownloadResult>>> = None;

#[async_trait::async_trait]
impl<T> Downloader for T
where
    T: Downloadable,
{
    type Data = T::Out;

    async fn download(&self, channel: Sender<Self::Data>) {
        let result = self.download().await;
        let _ = channel.send(result).await;
    }
}
