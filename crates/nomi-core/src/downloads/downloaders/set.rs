use std::fmt::Debug;

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

impl Debug for DownloadSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadSet").finish()
    }
}

impl DownloadSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_vec_dyn(vec: Vec<Box<dyn Downloadable<Out = DownloadResult>>>) -> Self {
        Self { set: vec }
    }

    pub fn add<D>(&mut self, downloader: Box<D>) -> &mut Self
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

    fn total(&self) -> u32 {
        self.set.len() as u32
    }

    async fn download(mut self: Box<Self>, channel: Sender<Self::Data>) {
        let mut set = JoinSet::new();

        for downloader in self.set {
            set.spawn(downloader.download());
        }

        while let Some(result) = set.join_next().await {
            let _ = match result {
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
