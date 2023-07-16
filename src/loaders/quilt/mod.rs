use async_trait::async_trait;
use log::{debug, info};
use reqwest::Client;

use crate::utils::GetPath;

use self::quilt_meta::{QuiltMeta, QuiltProfile, QuiltVersion};

use super::{maven::MavenData, Loader};

pub mod quilt_meta;

pub struct QuiltLoader {
    pub meta: QuiltMeta,
    pub game_version: String,

    pub profile: QuiltProfile,
}

pub struct QuiltVersions {
    pub latest_stable: QuiltVersion,
    pub all: QuiltMeta,
}

impl QuiltLoader {
    pub async fn get_versions() -> anyhow::Result<QuiltVersions> {
        let resp: QuiltMeta = Client::new()
            .get("https://meta.quiltmc.org/v3/versions/loader")
            .send()
            .await?
            .json()
            .await?;

        let latest = match resp.iter().find(|x| !x.version.contains("beta")) {
            Some(version) => version,
            None => &resp[0],
        };

        debug!("{:#?}", &latest);

        Ok(QuiltVersions {
            latest_stable: latest.clone(),
            all: resp,
        })
    }

    /// Set `quilt_version` to `None` for last stable version
    pub async fn new(game_version: &str, quilt_version: Option<&str>) -> anyhow::Result<Self> {
        let versions = Self::get_versions().await?;

        let quilt = if let Some(ver) = quilt_version {
            versions
                .all
                .iter()
                .find(|x| x.version == ver)
                .unwrap_or(&versions.latest_stable)
        } else {
            &versions.latest_stable
        };

        let response: QuiltProfile = Client::new()
            .get(format!(
                "https://meta.quiltmc.org/v3/versions/loader/{}/{}/profile/json",
                game_version, quilt.version
            ))
            .send()
            .await?
            .json()
            .await?;

        info!("{:#?}", &response);

        Ok(Self {
            meta: versions.all,
            game_version: game_version.to_string(),
            profile: response,
        })
    }

    pub async fn download_libraries(&self) -> anyhow::Result<()> {
        for i in self.profile.libraries.iter() {
            let maven = MavenData::new(i.name.as_str());

            info!(
                "{}{}{}",
                &i.url[0..i.url.len() - 1],
                maven.url,
                maven.url_file
            );

            self.dowload_file(
                GetPath::libraries()
                    .join(maven.local_file_path)
                    .join(maven.local_file),
                format!(
                    "{}{}/{}",
                    &i.url[0..i.url.len() - 1],
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
impl Loader for QuiltLoader {
    async fn download(&self) -> anyhow::Result<()> {
        let loader = self
            .profile
            .libraries
            .iter()
            .find(|i| i.name.contains("quilt-loader"))
            // we realy need it
            .unwrap();
        let maven = MavenData::new(loader.name.as_str());

        info!(
            "{}{}/{}",
            &loader.url[0..loader.url.len() - 1],
            maven.url,
            maven.url_file
        );

        self.dowload_file(
            // FIXME
            GetPath::versions()
                .join(&self.profile.id)
                .join(format!("{}.jar", &self.profile.id)),
            format!(
                "{}{}/{}",
                &loader.url[0..loader.url.len() - 1],
                maven.url,
                maven.url_file
            ),
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
