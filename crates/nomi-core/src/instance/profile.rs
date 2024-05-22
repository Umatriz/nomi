use serde::{Deserialize, Serialize};

use crate::repository::{simple_args::SimpleArgs, simple_lib::SimpleLib};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Loader {
    pub name: String,
    pub main_class: String,
    pub args: SimpleArgs,
    pub libraries: Vec<SimpleLib>,
}
