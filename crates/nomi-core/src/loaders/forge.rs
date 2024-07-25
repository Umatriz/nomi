use std::fmt::Debug;

use anyhow::anyhow;
use itertools::Itertools;

const FORGE_REPO_URL: &str = "https://maven.minecraftforge.net";
const FORGE_GROUP: &str = "net.minecraftforge";
const FORGE_ARTIFACT: &str = "forge";

const NEO_FORGE_REPO_URL: &str = "https://maven.neoforged.net/releases/";
const NEO_FORGE_GROUP: &str = "net.neoforged";
const NEO_FORGE_ARTIFACT: &str = "neoforge";

pub struct Forge {
    url: String,
}

impl Forge {
    #[tracing::instrument(skip_all, err)]
    pub async fn get_versions(game_version: impl Into<String>) -> anyhow::Result<Vec<String>> {
        let game_version = game_version.into();

        let raw = reqwest::get(format!("{FORGE_REPO_URL}/net/minecraftforge/forge/maven-metadata.xml"))
            .await?
            .text()
            .await?;

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

    pub fn new(version: impl Into<String>, forge_version: Option<impl Into<String>>) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_versions_test() {
        let versions = Forge::get_versions("1.19.2").await.unwrap();
        println!("{versions:#?}");
    }
}
