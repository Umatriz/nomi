use async_trait::async_trait;
use reqwest::Client;

use crate::utils::GetPath;

use super::{
    fabric_meta::{Meta, VersionLoader},
    maven::MavenData,
    Loader, FABRIC_MAVEN,
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

        let latest = response.iter().find(|i| i.loader.stable);

        println!("{:#?}", latest.unwrap());

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
                    "{}{}{}",
                    {
                        let mut url = i.url.clone().unwrap();
                        url.pop();

                        url
                    },
                    maven.url,
                    maven.url_file
                ),
            )
            .await?;
        }
        Ok(())
    }

    pub async fn download_intermediary(&self) -> anyhow::Result<()> {
        let maven = MavenData::new(self.latest.intermediary.maven.as_str());

        self.dowload_file(
            GetPath::libraries()
                .join(maven.local_file_path)
                .join(maven.local_file),
            format!("{}{}{}", FABRIC_MAVEN, maven.url, maven.url_file),
        )
        .await?;

        Ok(())
    }

    pub async fn create_json(&self) {
        todo!()
    }
}

#[async_trait(?Send)]
impl Loader for FabricLoader {
    async fn download(&self) -> anyhow::Result<()> {
        let maven = MavenData::new(self.latest.loader.maven.as_str());

        self.dowload_file(
            GetPath::versions().join(maven.local_file),
            format!("{}{}{}", FABRIC_MAVEN, maven.url, maven.url_file),
        )
        .await?;

        self.download_libraries().await?;
        self.download_intermediary().await?;

        Ok(())
    }

    fn create_json() -> anyhow::Result<()> {
        Ok(())
    }
}
