use anyhow::Result;
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tracing::info;

use crate::{
    calculate_sha1,
    downloads::{
        downloaders::file::FileDownloader,
        progress::ProgressSender,
        set::DownloadSet,
        traits::{DownloadResult, Downloadable, Downloader},
    },
    fs::write_json_config,
    PinnedFutureWithBounds,
};

use super::DownloadQueue;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Assets {
    pub objects: HashMap<String, AssetInformation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetInformation {
    pub hash: String,
    pub size: i64,
}

#[derive(Debug)]
pub struct AssetsDownloader {
    queue: DownloadQueue,
    assets: Assets,
    indexes: PathBuf,
    id: String,
}

#[derive(Debug)]
pub struct Chunk {
    ok: usize,
    err: usize,
    set: DownloadSet,
}

impl Chunk {
    pub fn new(set: DownloadSet) -> Self {
        Self { ok: 0, err: 0, set }
    }
}

#[async_trait::async_trait]
impl Downloader for Chunk {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.set.total()
    }

    async fn download(mut self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        let (helper_sender, mut helper_receiver) = tokio::sync::mpsc::channel(100);
        let downloader = self.set.with_helper(helper_sender);
        Box::new(downloader).download(sender).await;
        while let Some(result) = helper_receiver.recv().await {
            match result.0 {
                Ok(_) => self.ok += 1,
                Err(_) => self.err += 1,
            }
        }
        info!("Downloaded Chunk OK: {} ERR: {}", self.ok, self.err);
    }
}

impl AssetsDownloader {
    pub async fn new(url: String, id: String, objects: PathBuf, indexes: PathBuf) -> Result<Self> {
        let assets: Assets = Client::new().get(&url).send().await?.json().await?;

        let mut queue = DownloadQueue::new();

        assets
            .objects
            .iter()
            .collect_vec()
            .chunks(100)
            .map(|c| c.iter().map(|(_, v)| v).copied())
            .map(|chunk| {
                chunk
                    .filter_map(|asset| {
                        let path = objects.join(&asset.hash[0..2]).join(&asset.hash);

                        if path.exists() && std::fs::read(&path).ok().is_some_and(|buff| asset.hash == calculate_sha1(buff)) {
                            None
                        } else {
                            let downloader = FileDownloader::new(
                                format!("https://resources.download.minecraft.net/{}/{}", &asset.hash[0..2], asset.hash),
                                path,
                            )
                            .with_sha1(asset.hash.clone())
                            .into_retry();

                            Some(downloader)
                        }
                    })
                    .map::<Box<dyn Downloadable<Out = DownloadResult>>, _>(|downloader| Box::new(downloader))
                    .collect::<Vec<_>>()
            })
            .map(DownloadSet::from_vec_dyn)
            .map(Chunk::new)
            .for_each(|downloader| queue.add_downloader(downloader));

        Ok(Self { queue, assets, indexes, id })
    }
}

#[async_trait::async_trait]
impl Downloader for AssetsDownloader {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.queue.total()
    }

    #[tracing::instrument(skip_all)]
    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        Box::new(self.queue).download(sender).await;
    }

    fn io(&self) -> PinnedFutureWithBounds<anyhow::Result<()>> {
        let id = self.id.clone();
        let indexes = self.indexes.clone();
        let assets = self.assets.clone();

        let fut = async move {
            let path = indexes.join(format!("{id}.json"));
            write_json_config(&assets, path).await
        };

        Box::pin(fut)
    }
}
