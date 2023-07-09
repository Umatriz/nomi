use async_trait::async_trait;
use reqwest::Client;

use super::{
    fabric_meta::{Meta, VersionLoader},
    Loader,
};

pub struct FabricLoader {
    pub latest: VersionLoader,
    pub meta: Meta,
}

impl FabricLoader {
    pub async fn new(version: &str) -> anyhow::Result<Self> {
        let response: Meta = Client::new()
            .get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}",
                version
            ))
            .send()
            .await?
            .json()
            .await?;

        let latest = response.0.iter().find(|i| i.loader.stable);

        Ok(Self {
            meta: response.clone(),
            latest: latest.unwrap().clone(),
        })
    }

    fn download_libraries(&self) -> anyhow::Result<()> {
        todo!()
    }
}

#[async_trait(?Send)]
impl Loader for FabricLoader {
    async fn download(&self) -> anyhow::Result<()> {
        todo!()
    }

    fn create_json() -> anyhow::Result<()> {
        Ok(())
    }
}
