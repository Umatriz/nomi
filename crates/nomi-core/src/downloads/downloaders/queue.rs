use std::fmt::Debug;

use crate::downloads::{
    progress::ProgressSender,
    traits::{DownloadResult, Downloader},
};

#[derive(Default)]
pub struct DownloadQueue {
    queue: Vec<Box<dyn Downloader<Data = DownloadResult>>>,
    inspector: Option<Box<dyn Fn() + Sync + Send>>,
}

impl Debug for DownloadQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadQueue").finish()
    }
}

impl DownloadQueue {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_inspector<I: Fn() + Sync + Send + 'static>(mut self, inspector: I) -> Self {
        self.inspector = Some(Box::new(inspector));
        self
    }

    pub fn add_downloader<D>(&mut self, downloader: D)
    where
        D: Downloader<Data = DownloadResult> + 'static,
    {
        self.queue.push(Box::new(downloader));
    }

    #[must_use]
    pub fn with_downloader<D>(mut self, downloader: D) -> Self
    where
        D: Downloader<Data = DownloadResult> + 'static,
    {
        self.queue.push(Box::new(downloader));
        self
    }

    #[must_use]
    pub fn with_downloader_dyn(
        mut self,
        downloader: Box<dyn Downloader<Data = DownloadResult>>,
    ) -> Self {
        self.queue.push(downloader);
        self
    }
}

#[async_trait::async_trait]
impl Downloader for DownloadQueue {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.queue.iter().map(|downloader| downloader.total()).sum()
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        for downloader in self.queue {
            downloader.download(sender).await;
            self.inspector.as_ref().inspect(|f| f());
        }
    }
}
