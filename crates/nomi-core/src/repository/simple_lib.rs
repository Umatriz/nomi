use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::loaders::maven::MavenData;

#[derive(Serialize, Deserialize, Debug)]
pub struct SimpleLib {
    pub jar: PathBuf,
}

impl From<MavenData> for SimpleLib {
    fn from(value: MavenData) -> Self {
        Self { jar: value.path }
    }
}
