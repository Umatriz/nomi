use async_trait::async_trait;
use reqwest::Client;

use crate::utils::GetPath;

use super::{
    fabric_meta::{FabricProfile, Meta},
    maven::MavenData,
    Loader, FABRIC_MAVEN,
};

pub struct FabricLoader {
    pub meta: Meta,
    pub version: String,

    pub profile: FabricProfile,
}

// TODO: define `retry()` method

impl FabricLoader {
    pub async fn new(version: &str) -> anyhow::Result<Self> {
        let meta_response: Meta = Client::new()
            .get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}",
                version
            ))
            .send()
            .await?
            .json()
            .await?;

        let latest = match meta_response.iter().find(|i| i.loader.stable) {
            Some(last) => last,
            None => &meta_response[0],
        };

        println!("{:#?}", latest);

        let profile_reponse: FabricProfile = Client::new()
            .get(format!(
                "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                version, latest.loader.version
            ))
            .send()
            .await?
            .json()
            .await?;

        println!("{:#?}", profile_reponse);

        Ok(Self {
            meta: meta_response.clone(),
            // latest: latest.clone(),
            version: version.to_string(),
            profile: profile_reponse,
        })
    }

    pub async fn download_libraries(&self) -> anyhow::Result<()> {
        for i in self.profile.libraries.iter() {
            let maven = MavenData::new(&i.name);

            self.dowload_file(
                GetPath::libraries()
                    .join(maven.local_file_path)
                    .join(maven.local_file),
                format!(
                    "{}{}{}",
                    {
                        // FIXME
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
}

#[async_trait(?Send)]
impl Loader for FabricLoader {
    async fn download(&self) -> anyhow::Result<()> {
        let maven = MavenData::new(
            self.profile
                .libraries
                .iter()
                .find(|i| i.name.contains("fabric-loader"))
                // we realy need it
                .unwrap()
                .name
                .as_str(),
        );

        self.dowload_file(
            // FIXME
            GetPath::versions()
                .join(&self.profile.id)
                .join(format!("{}.jar", &self.profile.id)),
            format!("{}{}{}", FABRIC_MAVEN, maven.url, maven.url_file),
        )
        .await?;

        self.create_json()?;

        self.download_libraries().await?;

        Ok(())
    }

    fn create_json(&self) -> anyhow::Result<()> {
        let file_name = format!("{}.json", self.profile.id);

        let path = GetPath::versions().join(&self.profile.id).join(file_name);

        let _ = std::fs::create_dir_all(path.parent().unwrap());

        let file = std::fs::File::create(path)?;

        Ok(serde_json::to_writer_pretty(&file, &self.profile)?)
    }
}
