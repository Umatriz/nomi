use std::{fmt::Display, path::PathBuf, sync::LazyLock};

use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Default)]
pub struct MavenData {
    pub url: String,
    pub path: PathBuf,
    pub file_name: String,
}

impl MavenData {
    #[must_use]
    pub fn new(artifact: &str) -> Self {
        let artifact = MavenArtifact::new(artifact);
        Self::from_artifact_data(&artifact)
    }

    #[must_use]
    pub fn from_artifact_data(artifact: &MavenArtifact) -> Self {
        let group_parts = artifact.group.split('.').collect_vec();

        let classifier = if artifact.classifier.is_empty() {
            String::new()
        } else {
            format!("-{}", artifact.classifier)
        };

        let file_name = format!("{}-{}{classifier}.{}", &artifact.artifact, &artifact.version, &artifact.extension);

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
    pub classifier: String,
    pub extension: String,
}

impl MavenArtifact {
    #[must_use]
    pub fn new(artifact: &str) -> Self {
        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
            // PANICS: This will never panic because the pattern is valid.
            Regex::new(r"(?P<group>[^:]*):(?P<artifact>[^:]*):(?P<version>[^@:]*)(?::(?P<classifier>.*))?(?:@(?P<extension>.*))?").unwrap()
        });

        REGEX.captures(artifact).map_or_else(
            || {
                error!(artifact, "No values captured. Using provided artifact as a group");
                MavenArtifact {
                    group: artifact.to_string(),
                    artifact: String::new(),
                    version: String::new(),
                    classifier: String::new(),
                    extension: String::from("jar"),
                }
            },
            |captures| {
                let get_group = |name, default| captures.name(name).map_or(String::from(default), |v| String::from(v.as_str()));

                let group = get_group("group", "");
                let artifact = get_group("artifact", "");
                let version = get_group("version", "");
                let classifier = get_group("classifier", "");
                let extension = get_group("extension", "jar");

                MavenArtifact {
                    group,
                    artifact,
                    version,
                    classifier,
                    extension,
                }
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
        let artifact = MavenArtifact::new("de.oceanlabs.mcp:mcp_config:1.20.1-20230612.114412@zip");

        println!("{artifact:#?}");
    }

    #[test]
    fn parse_test() {
        let maven = MavenData::from_artifact_data(&MavenArtifact::new("net.fabricmc:fabric-loader:0.14.22"));
        assert_eq!(maven.path, PathBuf::from("net/fabricmc/fabric-loader/0.14.22/fabric-loader-0.14.22.jar"));

        let maven = MavenData::from_artifact_data(&MavenArtifact::new("de.oceanlabs.mcp:mcp_config:1.20.1-20230612.114412@zip"));
        assert_eq!(
            maven.path,
            PathBuf::from("de/oceanlabs/mcp/mcp_config/1.20.1-20230612.114412/mcp_config-1.20.1-20230612.114412.zip")
        );
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
