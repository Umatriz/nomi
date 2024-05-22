use std::path::PathBuf;

use itertools::Itertools;

#[derive(Debug, Default)]
pub struct MavenData {
    pub url: String,
    pub path: PathBuf,
    pub file: String,
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
            file: name,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env::current_dir;

    use crate::downloads::download_file;

    use super::*;

    #[test]
    fn parse_test() {
        let artifact = "net.fabricmc:fabric-loader:0.14.22";

        let maven = MavenData::new(artifact);
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
