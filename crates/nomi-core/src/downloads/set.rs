use tokio::{
    sync::{mpsc::Sender, Mutex},
    task::JoinSet,
};

use super::{
    downloadable::{DownloadResult, Downloadable, Downloader},
    DownloadError,
};

pub struct DownloadSet {
    set: Mutex<JoinSet<DownloadResult>>,
}

impl Default for DownloadSet {
    fn default() -> Self {
        Self {
            set: Mutex::new(JoinSet::new()),
        }
    }
}

impl DownloadSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add<D>(&mut self, downloader: D) -> &mut Self
    where
        D: Downloadable<Out = DownloadResult> + 'static,
    {
        {
            let mut set = self.set.lock().await;
            set.spawn(async move { downloader.download().await });
        }
        self
    }
}

#[async_trait::async_trait]
impl Downloader for DownloadSet {
    type Data = DownloadResult;

    async fn download(&self, channel: Sender<Self::Data>) {
        let mut set = self.set.lock().await;
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
