use std::{fmt::Display, path::PathBuf};

use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

#[derive(Debug, Default)]
pub struct MavenData {
    pub url: String,
    pub path: PathBuf,
    pub file_name: String,
}

impl MavenData {
    #[must_use]
    pub fn new(artifact: &str) -> Self {
        let mut chunks = artifact.split(':').map(|i| vec![i]).collect_vec();
        let group = chunks[0][0];
        let _ = std::mem::replace(&mut chunks[0], group.split('.').collect_vec());

        let mut flatten = chunks.iter().flatten().copied().collect_vec();

        let name = format!("{}-{}.jar", chunks[1][0], chunks[2][0]);
        flatten.push(&name);

        let path_iter = flatten.iter().copied();
        let path = itertools::intersperse(path_iter, "/").collect::<String>();

        Self {
            path: PathBuf::from(&path),
            url: urlencoding::encode(&path).into_owned(),
            file_name: name,
        }
    }

    #[must_use]
    pub fn from_artifact_data(artifact: &MavenArtifact) -> Self {
        let group_parts = artifact.group.split('.').collect_vec();
        let file_name = format!("{}-{}.jar", &artifact.artifact, &artifact.version);

        let path = group_parts
            .into_iter()
            .chain([artifact.artifact.as_str(), artifact.version.as_str(), file_name.as_str()])
            .join("/");

        let url = urlencoding::encode(&path).into_owned();

        Self {
            url,
            path: PathBuf::from(path),
            file_name,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MavenArtifact {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

impl MavenArtifact {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(artifact: &str) -> Self {
        // PANICS: This will never panic because the pattern is valid.
        let regex = Regex::new(r"(?P<group>.*):(?P<artifact>.*):(?P<version>.*)").unwrap();
        regex.captures(artifact).map_or_else(
            || {
                error!(artifact, "No values captured. Using provided artifact as a group");
                MavenArtifact {
                    group: artifact.to_string(),
                    artifact: String::new(),
                    version: String::new(),
                }
            },
            |captures| {
                let get_group = |name| captures.name(name).map_or(String::default(), |v| String::from(v.as_str()));

                let group = get_group("group");
                let artifact = get_group("artifact");
                let version = get_group("version");

                MavenArtifact { group, artifact, version }
            },
        )
    }
}

impl Display for MavenArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}:{}", self.group, self.artifact, self.version))
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use crate::downloads::download_file;

    use super::*;

    #[test]
    fn maven_artifact_parse_test() {
        let artifact = MavenArtifact::new("net.fabricmc:fabric-loader:0.14.22");

        println!("{:#?}", artifact);
    }

    #[test]
    fn parse_test() {
        let artifact = "net.fabricmc:fabric-loader:0.14.22";

        let maven = MavenData::from_artifact_data(&MavenArtifact::new(artifact));

        assert_eq!(maven.path, PathBuf::from("net/fabricmc/fabric-loader/0.14.22/fabric-loader-0.14.22.jar"));

        println!("{maven:#?}");
    }

    #[tokio::test]
    async fn get_test() {
        let artifact = "net.fabricmc:fabric-loader:0.14.22";

        let maven = MavenData::new(artifact);

        download_file(
            current_dir().unwrap().join(maven.path),
            format!("https://maven.fabricmc.net/{}", maven.url),
        )
        .await
        .unwrap();
    }
}
