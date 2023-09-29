use serde::{Deserialize, Serialize};

pub type FabricVersions = Vec<Version>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Version {
    loader: VersionLoader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionLoader {
    separator: String,
    build: i32,
    maven: String,
    version: String,
    stable: bool,
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
