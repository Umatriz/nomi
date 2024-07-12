use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::DynError;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub move_folders: Option<BTreeMap<PathBuf, PathBuf>>,
}

impl Config {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, DynError> {
        let s = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&s)?)
    }
}
