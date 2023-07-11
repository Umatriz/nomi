use async_trait::async_trait;
use reqwest::Client;

use crate::utils::GetPath;

use super::{
    fabric_meta::{Meta, VersionLoader},
    maven::MavenData,
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

    pub async fn download_libraries(&self) -> anyhow::Result<()> {
        for i in self.latest.launcher_meta.libraries.common.iter() {
            let maven = MavenData::new(&i.name);

            self.dowload_file(
                GetPath::libraries()
                    .join(maven.local_file_path)
                    .join(maven.local_file),
                format!(
                    "{}{}",
                    {
                        let mut url = String::new();
                        if let Some(i) = i.url.chars().collect::<Vec<_>>().pop() {
                            url.push(i)
                        }

                        url
                    },
                    maven.url_file
                ),
            )
            .await?;
        }
        Ok(())
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
