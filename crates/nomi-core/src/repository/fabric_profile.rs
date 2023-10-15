use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{instance::profile::Profile, loaders::maven::MavenData};

use super::{simple_args::SimpleArgs, simple_lib::SimpleLib};

#[derive(Serialize, Deserialize, Debug)]
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

impl Profile for FabricProfile {
    fn name(&self) -> String {
        self.id.clone()
    }

    fn main_class(&self) -> String {
        self.main_class.clone()
    }

    fn arguments(&self) -> super::simple_args::SimpleArgs {
        SimpleArgs::from(self.arguments.clone())
    }

    fn libraries(&self) -> Vec<super::simple_lib::SimpleLib> {
        self.libraries
            .iter()
            .map(|l| MavenData::new(&l.name))
            .map(SimpleLib::from)
            .collect_vec()
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Arguments {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub name: String,
    pub url: String,
}
