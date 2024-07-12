use reqwest::get;
use serde::{Deserialize, Serialize};

pub type FabricVersions = Vec<Version>;

pub async fn get_fabric_versions(game_version: String) -> anyhow::Result<FabricVersions> {
    get(format!("https://meta.fabricmc.net/v2/versions/loader/{game_version}"))
        .await?
        .json()
        .await
        .map_err(Into::into)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    pub loader: VersionLoader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionLoader {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[cfg(test)]
mod tests {
    use reqwest::get;

    use super::*;

    #[tokio::test]
    async fn deserialize_test() {
        let data = get("https://meta.fabricmc.net/v2/versions/loader/1.18.2")
            .await
            .unwrap()
            .json::<FabricVersions>()
            .await
            .unwrap();

        dbg!(&data[0..5]);
    }
}
