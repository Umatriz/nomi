use anyhow::{Context, Result};
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::{sync::mpsc::Sender, task::JoinSet};
use tracing::{error, info, trace};

use crate::{
    downloads::{downloadable::Downloadable, downloaders::AssetDownloader},
    utils::write_into_file,
};

use super::{
    download_file,
    downloadable::{DownloadResult, Downloader},
    set::DownloadSet,
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
    sender: Sender<DownloadResult>,
    assets: Assets,
    id: String,
    url: String,
}

impl AssetsDownloader {
    pub async fn new(url: String, id: String, sender: Sender<DownloadResult>) -> Result<Self> {
        Ok(Self {
            assets: Self::init(&url).await?,
            id,
            url,
            sender,
        })
    }

    async fn init(url: &str) -> Result<Assets> {
        let data: Assets = Client::new()
            .get(url)
            .send()
            .await
            .context("failed to send get request")?
            .json()
            .await
            .context("failed to parse json")?;

        Ok(data)
    }

    async fn create_dir(&self, main_dir: &Path, asset_dir_name: &str) -> Result<PathBuf> {
        let path = main_dir.join(asset_dir_name);

        tokio::fs::create_dir_all(&path).await?;

        Ok(path)
    }

    pub async fn get_assets_json(&self, assets_dir: &Path) -> Result<()> {
        let filen = format!("{}.json", self.id);
        let path = assets_dir.join(filen);

        let body = Client::new().get(&self.url).send().await?.text().await?;

        write_into_file(body.as_bytes(), &path).await
    }

    pub async fn download_assets_chunked(&self, dir: &Path) -> anyhow::Result<()> {
        async fn create_dir(main_dir: &Path, asset_hash: &str) -> anyhow::Result<PathBuf> {
            let path = main_dir.join(&asset_hash[0..2]);

            tokio::fs::create_dir_all(&path).await?;

            Ok(path.join(asset_hash))
        }

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
                let path = create_dir(dir, &asset.hash).await?;

                if path.exists() {
                    continue;
                }

                let url = format!(
                    "https://resources.download.minecraft.net/{}/{}",
                    &asset.hash[0..2],
                    asset.hash
                );

                download_set.add(AssetDownloader::new(url, path)).await;
            }

            download_set.download(self.sender.clone()).await;

            info!("Assets chunk downloaded");
        }

        Ok(())
    }

    // pub async fn download_assets_chunked(&self, dir: &Path) -> anyhow::Result<()> {
    //     let assets_chunks = self
    //         .assets
    //         .objects
    //         .iter()
    //         .collect_vec()
    //         .chunks(100)
    //         .map(|c| c.iter().cloned().collect())
    //         .collect::<Vec<HashMap<_, _>>>();

    //     // let assets_chunks = self.assets.objects.iter().chunks(100);

    //     // TODO: Implement retry pull for missing assets

    //     let mut retry = JoinSet::new();
    //     for asset in &assets_chunks {
    //         // let assets: Vec<_> = asset.collect();

    //         let mut set = JoinSet::new();
    //         for v in asset.values() {
    //             let path = self.create_dir(dir, &v.hash[0..2]).await?;
    //             let asset_path = path.join(&v.hash);

    //             if asset_path.exists() {
    //                 continue;
    //             }

    //             let url = format!(
    //                 "https://resources.download.minecraft.net/{}/{}",
    //                 &v.hash[0..2],
    //                 v.hash
    //             );

    //             set.spawn(download_file(asset_path, url));
    //         }

    //         let mut ok_assets = 0;
    //         let mut err_assets = 0;

    //         while let Some(res) = set.join_next().await {
    //             // FIXME
    //             let result = res?;
    //             if let Err(err) = result {
    //                 match err {
    //                     crate::downloads::DownloadError::Error { url, path, error } => {
    //                         error!("Downloading error: {}", error);
    //                         retry.spawn(download_file(path, url));
    //                     }
    //                     crate::downloads::DownloadError::JoinError(_) => continue,
    //                 }
    //                 err_assets += 1;
    //             } else {
    //                 ok_assets += 1;
    //             }
    //         }

    //         info!(
    //             "Chunk downloaded: OK - {}; MISSING - {}",
    //             ok_assets, err_assets
    //         );
    //     }

    //     while let Some(res) = retry.join_next().await {
    //         res??
    //     }

    //     Ok(())
    // }
}
