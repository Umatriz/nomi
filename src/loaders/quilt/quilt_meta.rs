use serde::{Deserialize, Serialize};

use crate::loaders::profile::LoaderProfile;

pub type QuiltMeta = Vec<QuiltVersion>;

/// https://meta.quiltmc.org/v3/versions/loader

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuiltVersion {
    pub separator: String,
    pub build: i32,
    pub maven: String,
    pub version: String,
}

/// https://meta.quiltmc.org/v3/versions/loader/:game_version/:loader_version/profile/json

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuiltProfile {
    pub id: String,
    pub inherits_from: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub main_class: String,
    pub arguments: QuiltArguments,
    pub libraries: Vec<QuiltLibrary>,
    pub release_time: String,
    pub time: String,
}

impl LoaderProfile for QuiltProfile {
    fn get_args(&self) -> crate::loaders::profile::LoaderProfileArguments {
        crate::loaders::profile::LoaderProfileArguments {
            game: Some(self.arguments.game.clone()),
            jvm: None,
        }
    }

    fn get_main_class(&self) -> String {
        self.main_class.clone()
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuiltArguments {
    pub game: Vec<String>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuiltLibrary {
    pub name: String,
    pub url: String,
}
