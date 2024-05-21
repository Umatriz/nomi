use tokio::{sync::mpsc::Sender, task::JoinSet};

use crate::downloads::{
    traits::{DownloadResult, Downloadable, Downloader},
    DownloadError,
};

/// Downloader that starts downloading all provided [`Downloadable`] elements
/// when [`Downloader::download`] is called
#[derive(Default)]
pub struct DownloadSet {
    set: Vec<Box<dyn Downloadable<Out = DownloadResult>>>,
}

impl DownloadSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add<D>(&mut self, downloader: Box<D>) -> &mut Self
    where
        D: Downloadable<Out = DownloadResult> + 'static,
    {
        self.set.push(downloader);
        self
    }
}

#[async_trait::async_trait]
impl Downloader for DownloadSet {
    type Data = DownloadResult;

    async fn download(mut self: Box<Self>, channel: Sender<Self::Data>) {
        let mut set = JoinSet::new();

        for downloader in self.set {
            set.spawn(downloader.download());
        }

        while let Some(result) = set.join_next().await {
            let _ = match dbg!(result) {
                Ok(download_status) => channel.send(download_status).await,
                Err(join_error) => {
                    channel
                        .send(Err(DownloadError::JoinError(join_error)))
                        .await
                }
            };
        }
    }
}
