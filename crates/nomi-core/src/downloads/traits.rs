use egui_task_manager::Progress;

use super::{downloaders::assets::AssetsDownloaderIo, progress::ProgressSender, DownloadError};

#[derive(Debug, Clone)]
pub struct DownloadResult(pub Result<DownloadStatus, DownloadError>);

impl Progress for DownloadResult {
    fn apply(&self, current: &mut u32) {
        *current += self.0.as_ref().map_or(0, |_| 1);
    }
}

#[must_use]
#[derive(Debug, Clone)]
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

    async fn download(self: Box<Self>) -> Self::Out;
}

const _: Option<Box<dyn Downloadable<Out = DownloadResult>>> = None;

#[async_trait::async_trait]
pub trait Downloader: Send + Sync {
    type Data;

    /// Returns the number of items to download
    fn total(&self) -> u32;
    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>);
}

const _: Option<Box<dyn Downloader<Data = DownloadResult>>> = None;

#[async_trait::async_trait]
impl<T> Downloader for T
where
    T: Downloadable,
{
    type Data = T::Out;

    fn total(&self) -> u32 {
        1
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        let result = self.download().await;
        sender.update(result).await;
    }
}

#[async_trait::async_trait]
pub trait DownloaderIO {
    async fn io(&self) -> anyhow::Result<()>;
}

const _: Option<Box<dyn DownloaderIO>> = None;

pub trait DownloaderIOExt<'a> {
    type IO: DownloaderIO;

    fn get_io(&'a self) -> Self::IO;
}

const _: Option<Box<dyn DownloaderIOExt<IO = AssetsDownloaderIo>>> = None;

pub trait ObjectSafeDownloaderIOExt<'a> {
    fn get_io_dyn(&'a self) -> Box<dyn DownloaderIO + Send + 'a>;
}

impl<'a, T: DownloaderIOExt<'a>> ObjectSafeDownloaderIOExt<'a> for T
where
    T::IO: Send,
{
    fn get_io_dyn(&'a self) -> Box<dyn DownloaderIO + Send + 'a> {
        Box::new(self.get_io())
    }
}
