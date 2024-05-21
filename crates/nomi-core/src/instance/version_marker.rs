use std::fmt::Debug;

use crate::{
    downloads::traits::{DownloadResult, Downloader, ObjectSafeDownloaderIOExt},
    loaders::{fabric::Fabric, vanilla::Vanilla},
};

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

impl Version for Vanilla {}
impl Version for Fabric {}
