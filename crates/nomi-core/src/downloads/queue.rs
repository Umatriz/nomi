use tokio::sync::mpsc::Sender;

use super::downloadable::{DownloadResult, Downloader};

#[derive(Default)]
pub struct DownloadQueue {
    queue: Vec<Box<dyn Downloader<Data = DownloadResult>>>,
}

impl DownloadQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<D>(&mut self, downloader: D) -> &mut Self
    where
        D: Downloader<Data = DownloadResult> + 'static,
    {
        self.queue.push(Box::new(downloader));
        self
    }
}

#[async_trait::async_trait]
impl Downloader for DownloadQueue {
    type Data = DownloadResult;

    async fn download(&self, channel: Sender<Self::Data>) {
        for downloader in &self.queue {
            downloader.download(channel.clone()).await;
        }
    }
}
