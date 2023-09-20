use anyhow::{Context, Result};
use log::trace;
use owo_colors::OwoColorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::{io::AsyncWriteExt, task::JoinSet};

use super::download_file;

#[derive(Serialize, Deserialize, Debug)]
pub struct Assets {
    pub objects: HashMap<String, AssetInformation>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetInformation {
    pub hash: String,
    pub size: i64,
}

#[derive(Debug)]
pub struct AssetsDownload {
    assets: Assets,
    id: String,
    url: String,
}

impl AssetsDownload {
    pub async fn new(url: String, id: String) -> Result<Self> {
        Ok(Self {
            assets: Self::init(url.clone()).await?,
            id,
            url,
        })
    }

    async fn init(url: String) -> Result<Assets> {
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
        let path = main_dir.join("assets").join("objects").join(asset_dir_name);

        tokio::fs::create_dir_all(&path).await?;

        trace!("Dir {} created successfully", path.to_string_lossy());

        Ok(path)
    }

    pub async fn get_assets_json(&self, assets_dir: &Path) -> Result<()> {
        let filen = format!("{}.json", self.id);
        let path = assets_dir.join("assets").join("indexes").join(filen);

        tokio::fs::create_dir_all(path.parent().context("")?).await?;

        let body = Client::new().get(&self.url).send().await?.text().await?;

        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(body.as_bytes()).await?;

        Ok(())
    }

    pub async fn download_assets(&self, dir: &Path) -> Result<()> {
        let mut set = JoinSet::new();

        for (_k, v) in self.assets.objects.iter() {
            let path = self.create_dir(dir, &v.hash[0..2]).await?;
            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                &v.hash[0..2],
                v.hash
            );

            set.spawn(download_file(path.join(&v.hash), url));
        }

        // TODO: Implement retry pull for missing assets

        while let Some(res) = set.join_next().await {
            let result = res.unwrap();
            if result.is_err() {
                let str = "MISSING ASSET".bright_red();
                log::error!("{}", str)
            }
        }

        Ok(())
    }
}
