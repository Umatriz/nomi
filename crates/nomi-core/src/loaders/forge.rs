use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
    slice::Iter,
};

use anyhow::anyhow;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    downloads::{
        download_file,
        progress::ProgressSender,
        traits::{DownloadResult, DownloadStatus, Downloader},
        FileDownloader,
    },
    PinnedFutureWithBounds, DOT_NOMI_TEMP_DIR,
};

const FORGE_REPO_URL: &str = "https://maven.minecraftforge.net";
const FORGE_GROUP: &str = "net.minecraftforge";
const FORGE_ARTIFACT: &str = "forge";

const NEO_FORGE_REPO_URL: &str = "https://maven.neoforged.net/releases/";
const NEO_FORGE_GROUP: &str = "net.neoforged";
const NEO_FORGE_ARTIFACT: &str = "neoforge";

/// Some versions require to have a suffix
const FORGE_SUFFIXES: &[(&str, &[&str])] = &[
    ("1.11", &["-1.11.x"]),
    ("1.10.2", &["-1.10.0"]),
    ("1.10", &["-1.10.0"]),
    ("1.9.4", &["-1.9.4"]),
    ("1.9", &["-1.9.0", "-1.9"]),
    ("1.8.9", &["-1.8.9"]),
    ("1.8.8", &["-1.8.8"]),
    ("1.8", &["-1.8"]),
    ("1.7.10", &["-1.7.10", "-1710ls", "-new"]),
    ("1.7.2", &["-mc172"]),
];

#[derive(Debug)]
pub struct Forge {
    urls: Vec<String>,
    game_version: String,
    forge_version: String,
}

impl Forge {
    #[tracing::instrument(skip_all, err)]
    pub async fn get_versions(game_version: impl Into<String>) -> anyhow::Result<Vec<String>> {
        let game_version = game_version.into();

        let raw = reqwest::get(format!("{FORGE_REPO_URL}/net/minecraftforge/forge/maven-metadata.xml"))
            .await?
            .text()
            .await?;

        // Parsing the XML to get versions list
        let versions = raw
            .find("<version>")
            .and_then(|s| raw.find("</versions>").map(|e| (s, e)))
            .map(|(s, e)| &raw[s..e])
            .map(|s| {
                s.split("<version>")
                    .filter_map(|s| {
                        s.split("</version>")
                            .collect::<String>()
                            .split('-')
                            .map(String::from)
                            .collect_tuple::<(String, String)>()
                    })
                    .filter(|(g, _)| g == &game_version)
                    .map(|(_, f)| f)
                    .collect::<Vec<_>>()
            });

        match versions {
            Some(v) => Ok(v),
            None => Err(anyhow!("Error while matching forge versions")),
        }
    }

    /// Get forge versions that are recommended for specific game version
    pub async fn get_promo_versions() -> anyhow::Result<ForgeVersions> {
        reqwest::get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")
            .await?
            .json::<ForgeVersions>()
            .await
            .map_err(Into::into)
    }

    #[tracing::instrument(skip(version), fields(game_version) err)]
    pub async fn new(version: impl Into<String>, forge_version: ForgeVersion) -> anyhow::Result<Self> {
        let game_version: String = version.into();

        tracing::Span::current().record("game_version", &game_version);

        let promo_versions = Self::get_promo_versions().await?;

        let from_promo = |version| {
            // Sometime one of those does not exist so we have a fallback.
            let next_version = match version {
                ForgeVersion::Recommended => ForgeVersion::Latest,
                ForgeVersion::Latest => ForgeVersion::Recommended,
                ForgeVersion::Specific(_) => unreachable!(),
            };

            promo_versions
                .promos
                .get(&version.format(&game_version))
                .or_else(|| promo_versions.promos.get(&next_version.format(&game_version)))
                .cloned()
                .map(ForgeVersion::Specific)
        };

        let opt = match forge_version {
            ForgeVersion::Specific(v) => Some(ForgeVersion::Specific(v)),
            version => from_promo(version),
        };

        let Some(ForgeVersion::Specific(forge_version)) = opt else {
            return Err(anyhow!("Cannot match version"));
        };

        let mut suffixes = vec![""];

        if let Some((_, s)) = FORGE_SUFFIXES.iter().find(|(k, _)| k == &game_version) {
            suffixes.extend(s.iter());
        }

        // Make list of urls that we should try to get installer from
        let urls = suffixes.into_iter().map(|suffix| {
            format!(
                "{FORGE_REPO_URL}/net/minecraftforge/forge/{game_version}-{forge_version}{suffix}/forge-{game_version}-{forge_version}{suffix}-installer.jar",
            )
        }).collect_vec();

        Ok(Self {
            urls,
            game_version,
            forge_version,
        })
    }

    pub fn installer_path(&self) -> PathBuf {
        Path::new(DOT_NOMI_TEMP_DIR).join(format!("{}-{}.jar", &self.game_version, &self.forge_version))
    }
}

#[async_trait::async_trait]
impl Downloader for Forge {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        1
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        let installer_path = self.installer_path();

        for url in &self.urls {
            if let Err(err) = dbg!(download_file(&installer_path, url).await) {
                warn!(
                    url = url,
                    error = ?err,
                    "Error while downloading Forge {}. Trying next suffix.",
                    &self.forge_version
                );
                continue;
            }

            // If the download is successful the break the loop
            sender.update(DownloadResult(Ok(DownloadStatus::Success))).await;
            break;
        }
    }

    fn io(&self) -> PinnedFutureWithBounds<anyhow::Result<()>> {
        async fn inner() -> anyhow::Result<()> {
            todo!()
        }

        Box::pin(inner())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ForgeVersions {
    pub homepage: String,
    pub promos: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ForgeVersion {
    Recommended,
    Latest,
    Specific(String),
}

impl ForgeVersion {
    pub fn format(&self, game_version: &str) -> String {
        format!("{game_version}-{}", self.as_str())
    }

    pub fn as_str(&self) -> &str {
        match self {
            ForgeVersion::Recommended => "recommended",
            ForgeVersion::Latest => "latest",
            ForgeVersion::Specific(v) => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_versions_test() {
        let versions = Forge::get_versions("1.7.10").await.unwrap();
        println!("{versions:#?}");
    }

    #[tokio::test]
    async fn create_forge_test() {
        let recommended = Forge::new("1.7.10", ForgeVersion::Recommended).await.unwrap();
        println!("{recommended:#?}");

        let latest = Forge::new("1.19.2", ForgeVersion::Latest).await.unwrap();
        println!("{latest:#?}");
    }

    #[tokio::test]
    async fn download_installer_test() {
        let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

        let recommended = Forge::new("1.7.10", ForgeVersion::Recommended).await.unwrap();
        println!("{recommended:#?}");

        dbg!(&recommended.urls[0]);

        let (tx, mut rx) = tokio::sync::mpsc::channel(5);

        Box::new(recommended).download(&tx).await;

        dbg!(rx.recv().await);

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
