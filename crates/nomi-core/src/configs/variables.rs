use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Variables {
    pub root: PathBuf,
}

impl Variables {
    pub fn is_current(&self) -> anyhow::Result<bool> {
        Ok(std::env::current_dir()? == self.root)
    }
}
