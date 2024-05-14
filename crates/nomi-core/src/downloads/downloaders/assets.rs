use anyhow::Result;
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::mpsc::Sender;
use tracing::info;

use crate::{
    downloads::{
        download_file,
        downloadable::{
            DownloadResult, DownloadStatus, Downloadable, Downloader, DownloaderIo, DownloaderIoExt,
        },
        downloaders::file::FileDownloader,
        set::DownloadSet,
    },
    utils::write_to_file,
};

#[derive(Serialize, Deserialize, Debug)]
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
    assets: Assets,
    objects: PathBuf,
    indexes: PathBuf,
    id: String,
    url: String,
}

impl AssetsDownloader {
    pub async fn new(url: String, id: String, objects: PathBuf, indexes: PathBuf) -> Result<Self> {
        let assets: Assets = Client::new().get(&url).send().await?.json().await?;

        Ok(Self {
            assets,
            objects,
            indexes,
            id,
            url,
        })
    }
}

impl<'a> DownloaderIoExt<'a, AssetsDownloaderIo<'a>> for AssetsDownloader {
    fn get_io(&'a self) -> AssetsDownloaderIo<'a> {
        AssetsDownloaderIo {
            assets: &self.assets,
            indexes: self.indexes.clone(),
            id: self.id.clone(),
        }
    }
}

pub struct AssetsDownloaderIo<'a> {
    assets: &'a Assets,
    indexes: PathBuf,
    id: String,
}

#[async_trait::async_trait]
impl DownloaderIo for AssetsDownloaderIo<'_> {
    async fn io(&self) -> anyhow::Result<()> {
        let path = self.indexes.join(format!("{}.json", self.id));

        let body = serde_json::to_string(&self.assets)?;

        write_to_file(body.as_bytes(), &path).await
    }
}

#[async_trait::async_trait]
impl Downloader for AssetsDownloader {
    type Data = DownloadResult;

    async fn download(&self, channel: Sender<Self::Data>) {
        let assets_chunks = self
            .assets
            .objects
            .iter()
            .collect_vec()
            .chunks(100)
            .map(|c| c.iter().map(|(_, v)| v).cloned().collect())
            .collect::<Vec<Vec<_>>>();

        for chunk in &assets_chunks {
            let mut download_set = DownloadSet::new();

            for asset in chunk {
                let path = self.objects.join(&asset.hash[0..2]).join(&asset.hash);

                if path.exists() {
                    continue;
                }

                let url = format!(
                    "https://resources.download.minecraft.net/{}/{}",
                    &asset.hash[0..2],
                    asset.hash
                );

                download_set.add(FileDownloader::new(url, path)).await;
            }

            download_set.download(channel.clone()).await;

            info!("Assets chunk downloaded");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::downloads::queue::DownloadQueue;

    use super::*;

    #[tokio::test]
    async fn assets_test() {
        let assets_downloader = AssetsDownloader::new(
            "url".into(),
            "id".into(),
            "objects".into(),
            "indexes".into(),
        )
        .await
        .unwrap();

        let io = assets_downloader.get_io();

        let set = DownloadQueue::new().add(assets_downloader);
    }
}
