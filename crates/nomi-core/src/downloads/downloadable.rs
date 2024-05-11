use tokio::sync::mpsc::Sender;

use super::DownloadError;

#[must_use]
pub enum DownloadStatus {
    /// Downloaded successfully
    Success,
    /// Downloaded successfully certain amount of elements
    SuccessWithProgress(u32),
    /// Error during downloading, must retry
    Error(DownloadError),
}

impl DownloadStatus {
    pub fn progress(progress: u32) -> Self {
        Self::SuccessWithProgress(progress)
    }
}

impl From<Result<(), DownloadError>> for DownloadStatus {
    fn from(value: Result<(), DownloadError>) -> Self {
        match value {
            Ok(_) => Self::Success,
            Err(err) => Self::Error(err),
        }
    }
}

#[async_trait::async_trait]
pub trait Downloadable: Send + Sync {
    type Out;

    async fn download(&self, channel: Sender<DownloadStatus>) -> Self::Out;
}

const _: Option<Box<dyn Downloadable<Out = ()>>> = None;
