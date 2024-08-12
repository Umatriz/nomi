use std::fmt::Debug;

use crate::{
    downloads::traits::{DownloadResult, Downloader},
    loaders::vanilla::Vanilla,
};

use super::{
    builder_ext::LaunchInstanceBuilderExt,
    launch::{LaunchInstanceBuilder, LaunchSettings},
};

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
