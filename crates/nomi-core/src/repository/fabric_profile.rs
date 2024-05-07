use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{instance::profile::LoaderProfile, utils::maven::MavenData};

use super::{simple_args::SimpleArgs, simple_lib::SimpleLib};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FabricProfile {
    pub id: String,
    pub inherits_from: String,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub main_class: String,
    pub arguments: Arguments,
    pub libraries: Vec<Library>,
}

impl From<FabricProfile> for LoaderProfile {
    fn from(val: FabricProfile) -> Self {
        LoaderProfile {
            name: val.id,
            main_class: val.main_class,
            args: SimpleArgs::from(val.arguments),
            libraries: val
                .libraries
                .iter()
                .map(|l| MavenData::new(&l.name))
                .map(SimpleLib::from)
                .collect_vec(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Arguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub name: String,
    pub url: String,
}
