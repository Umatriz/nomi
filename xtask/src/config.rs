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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn example_config_test() {
        let mut file = std::fs::File::create("./Xtask.toml").unwrap();
        let c = Config {
            move_folders: Some(BTreeMap::from([
                (
                    Path::new("./.nomi").to_path_buf(),
                    Path::new("./.nomi").to_path_buf(),
                ),
                (
                    Path::new("./.cfg").to_path_buf(),
                    Path::new("./.cfg").to_path_buf(),
                ),
            ])),
        };
        let data = toml::to_string_pretty(&c).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    }
}
