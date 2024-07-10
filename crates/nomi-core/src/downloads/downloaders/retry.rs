use crate::downloads::traits::{DownloadResult, RetryDownloader};

pub struct RetryPool {
    base: Box<dyn RetryDownloader<Out = DownloadResult, Data = DownloadResult>>,
}

impl RetryPool {
    pub fn new(
        base: Box<dyn RetryDownloader<Out = DownloadResult, Data = DownloadResult>>,
    ) -> Self {
        Self { base }
    }

    /// It will retry downloading failed files and return earlier if nothing failed.
    pub fn retry(iterations: usize) {}
}
