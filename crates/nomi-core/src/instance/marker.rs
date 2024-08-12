use std::fmt::Debug;

use crate::downloads::traits::{DownloadResult, Downloader};

use super::builder_ext::LaunchInstanceBuilderExt;

pub trait ProfileDownloader: LaunchInstanceBuilderExt + Downloader<Data = DownloadResult> + Debug + Send + Sync {
    fn into_downloader(self: Box<Self>) -> Box<dyn Downloader<Data = DownloadResult>>;
}

impl<T> ProfileDownloader for T
where
    T: LaunchInstanceBuilderExt + Downloader<Data = DownloadResult> + Debug + Send + Sync + 'static,
{
    fn into_downloader(self: Box<Self>) -> Box<dyn Downloader<Data = DownloadResult>> {
        self
    }
}
