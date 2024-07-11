use std::fmt::Debug;

use crate::downloads::traits::{DownloadResult, Downloader, ObjectSafeDownloaderIOExt};

use super::builder_ext::LaunchInstanceBuilderExt;

pub trait Version:
    LaunchInstanceBuilderExt + Downloader<Data = DownloadResult> + for<'a> ObjectSafeDownloaderIOExt<'a> + Debug + Send + Sync
{
    fn into_downloader(self: Box<Self>) -> Box<dyn Downloader<Data = DownloadResult>>;
}

impl<T> Version for T
where
    T: LaunchInstanceBuilderExt + Downloader<Data = DownloadResult> + for<'a> ObjectSafeDownloaderIOExt<'a> + Debug + Send + Sync + 'static,
{
    fn into_downloader(self: Box<Self>) -> Box<dyn Downloader<Data = DownloadResult>> {
        self
    }
}
