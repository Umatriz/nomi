use std::{
    collections::HashMap,
    fmt::Debug,
    io::Read,
    path::{Path, PathBuf},
    slice::Iter,
};

use anyhow::{anyhow, bail};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::{
    downloads::{
        download_file,
        progress::ProgressSender,
        traits::{DownloadResult, DownloadStatus, Downloader},
        FileDownloader,
    },
    repository::manifest::Library,
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
    profile: ForgeProfile,
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
    #[tracing::instrument(err)]
    pub async fn get_promo_versions() -> anyhow::Result<ForgeVersions> {
        reqwest::get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")
            .await?
            .json::<ForgeVersions>()
            .await
            .map_err(Into::into)
    }

    async fn proceed_version(game_version: &str, forge_version: ForgeVersion) -> Option<String> {
        let promo_versions = Self::get_promo_versions().await.ok()?;

        let from_promo = |version| {
            // Sometime one of those does not exist so we have a fallback.
            let next_version = match version {
                ForgeVersion::Recommended => ForgeVersion::Latest,
                ForgeVersion::Latest => ForgeVersion::Recommended,
                ForgeVersion::Specific(_) => unreachable!(),
            };

            promo_versions
                .promos
                .get(&version.format(game_version))
                .or_else(|| promo_versions.promos.get(&next_version.format(game_version)))
                .cloned()
        };

        match forge_version {
            ForgeVersion::Specific(v) => Some(v),
            version => from_promo(version),
        }
    }

    /// Get list of urls that we should try to get installer from
    fn get_urls(game_version: &str, forge_version: &str) -> Vec<String> {
        let mut suffixes = vec![""];

        if let Some((_, s)) = FORGE_SUFFIXES.iter().find(|(k, _)| k == &game_version) {
            suffixes.extend(s.iter());
        }

        suffixes.into_iter().map(|suffix| {
            format!(
                "{FORGE_REPO_URL}/net/minecraftforge/forge/{game_version}-{forge_version}{suffix}/forge-{game_version}-{forge_version}{suffix}-installer.jar",
            )
        }).collect_vec()
    }

    fn get_profile_from_installer(installer_path: &Path) -> anyhow::Result<ForgeProfile> {
        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let index = archive
            .index_for_name("version.json")
            .or_else(|| archive.index_for_name("install_profile.json"));

        let Some(idx) = index else {
            bail!("Cannot find either `version.json` or `install_profile.json`")
        };

        let mut file = archive.by_index(idx)?;

        let mut string = String::new();
        file.read_to_string(&mut string)?;

        serde_json::from_str(&string).map_err(Into::into)
    }

    #[tracing::instrument(skip(version), fields(game_version) err)]
    pub async fn new(version: impl Into<String>, forge_version: ForgeVersion) -> anyhow::Result<Self> {
        let game_version: String = version.into();

        tracing::Span::current().record("game_version", &game_version);

        let Some(forge_version) = Self::proceed_version(&game_version, forge_version).await else {
            bail!("Cannot match version");
        };

        let installer_path = forge_installer_path(&game_version, &forge_version);

        let urls = Self::get_urls(&game_version, &forge_version);
        for url in &urls {
            if let Err(err) = download_file(&installer_path, url).await {
                warn!(
                    url = url,
                    error = ?err,
                    "Error while downloading Forge {}. Trying next suffix.",
                    &forge_version
                );
                continue;
            }

            break;
        }

        let profile = Self::get_profile_from_installer(&installer_path)?;

        Ok(Self {
            profile,
            game_version,
            forge_version,
        })
    }

    pub fn installer_path(&self) -> PathBuf {
        forge_installer_path(&self.game_version, &self.forge_version)
    }
}

fn forge_installer_path(game_version: &str, forge_version: &str) -> PathBuf {
    Path::new(DOT_NOMI_TEMP_DIR).join(format!("{game_version}-{forge_version}.jar"))
}

#[async_trait::async_trait]
impl Downloader for Forge {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        1
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {}

    fn io(&self) -> PinnedFutureWithBounds<anyhow::Result<()>> {
        Box::pin(async { Ok(()) })
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ForgeProfile {
    New(Box<ForgeProfileNew>),
    Old(Box<ForgeProfileOld>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForgeProfileNew {
    #[serde(rename = "_comment_")]
    pub comment: Vec<String>,
    pub id: String,
    pub time: String,
    pub release_time: String,
    #[serde(rename = "type")]
    pub forge_profile_type: String,
    pub main_class: String,
    pub inherits_from: String,
    pub logging: Logging,
    pub arguments: crate::repository::manifest::Arguments,
    pub libraries: Vec<Library>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Logging {}

impl ForgeProfileNew {
    pub async fn download(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForgeProfileOld {
    // pub install: Install,
    pub version_info: VersionInfo,
}

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(rename_all = "camelCase")]
// pub struct Install {
//     pub profile_name: String,
//     pub target: String,
//     pub path: String,
//     pub version: String,
//     pub file_path: String,
//     pub welcome: String,
//     pub minecraft: String,
//     pub mirror_list: String,
//     pub logo: String,
// }

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub id: String,
    pub time: String,
    pub release_time: String,
    #[serde(rename = "type")]
    pub version_info_type: String,
    pub minecraft_arguments: String,
    pub main_class: String,
    pub minimum_launcher_version: i64,
    pub assets: String,
    pub inherits_from: String,
    pub jar: String,
    pub libraries: Vec<ForgeOldLibrary>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForgeOldLibrary {
    pub name: String,
    pub url: Option<String>,
    pub serverreq: Option<bool>,
    pub checksums: Option<Vec<String>>,
    pub clientreq: Option<bool>,
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

        let recommended = Forge::new("1.19.2", ForgeVersion::Recommended).await.unwrap();
        println!("{recommended:#?}");

        let io = recommended.io();

        if !recommended.installer_path().exists() {
            let (tx, mut rx) = tokio::sync::mpsc::channel(5);
            Box::new(recommended).download(&tx).await;
            dbg!(rx.recv().await);
        }

        io.await.unwrap();
    }
}
