use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use super::downloadable::{DownloadStatus, Downloadable};

#[derive(Default)]
pub struct DownloadQueue {
    queue: Vec<Arc<dyn Downloadable<Out = ()>>>,
}

impl DownloadQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<D>(&mut self, downloader: D) -> &mut Self
    where
        D: Downloadable<Out = ()> + 'static,
    {
        self.queue.push(Arc::new(downloader));
        self
    }
}

#[async_trait::async_trait]
impl Downloadable for DownloadQueue {
    type Out = ();

    async fn download(&self, channel: Sender<DownloadStatus>) -> Self::Out {
        for downloader in &self.queue {
            downloader.download(channel.clone()).await;
        }
    }
}
