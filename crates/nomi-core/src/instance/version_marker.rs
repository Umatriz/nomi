use std::fmt::Debug;

use crate::downloads::traits::{DownloadResult, Downloader, ObjectSafeDownloaderIOExt};

use super::builder_ext::LaunchInstanceBuilderExt;

pub trait Version:
    LaunchInstanceBuilderExt
    + Downloader<Data = DownloadResult>
    + for<'a> ObjectSafeDownloaderIOExt<'a>
    + Debug
    + Send
    + Sync
{
}

impl<T> Version for T where
    T: LaunchInstanceBuilderExt
        + Downloader<Data = DownloadResult>
        + for<'a> ObjectSafeDownloaderIOExt<'a>
        + Debug
        + Send
        + Sync
{
}
