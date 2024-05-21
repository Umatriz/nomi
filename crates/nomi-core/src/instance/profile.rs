use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::repository::{simple_args::SimpleArgs, simple_lib::SimpleLib};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoaderProfile {
    pub name: String,
    pub main_class: String,
    pub args: SimpleArgs,
    pub libraries: Vec<SimpleLib>,
}
