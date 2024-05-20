use std::{fmt::Debug, sync::Arc};

use crate::{
    downloads::downloadable::{
        DownloadResult, Downloader, DownloaderIO, DownloaderIOExt, ObjectSafeDownloaderIOExt,
    },
    loaders::{
        fabric::{Fabric, FabricIO},
        vanilla::{Vanilla, VanillaIO},
    },
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
