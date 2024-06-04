use serde::{Deserialize, Serialize};

use crate::{
    configs::profile::Loader,
    repository::{simple_args::SimpleArgs, simple_lib::SimpleLib},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoaderProfile {
    pub loader: Loader,
    pub main_class: String,
    pub args: SimpleArgs,
    pub libraries: Vec<SimpleLib>,
}
