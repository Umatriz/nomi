use log::{debug, info};
use reqwest::Client;

use self::quilt_meta::{QuiltMeta, QuiltProfile, QuiltVersion};

pub mod quilt_meta;

pub struct QuiltLoader {
    pub meta: QuiltMeta,
    pub version: String,

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

    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            meta: todo!(),
            version: todo!(),
            profile: todo!(),
        })
    }
}
