use std::fmt::Debug;

use crate::{
    downloads::download_version::DownloadVersion,
    loaders::{fabric::Fabric, vanilla::Vanilla},
};

use super::builder_ext::LaunchInstanceBuilderExt;

pub trait Version: LaunchInstanceBuilderExt + DownloadVersion + Debug + Send + Sync {}

impl Version for Vanilla {}
impl Version for Fabric {}
