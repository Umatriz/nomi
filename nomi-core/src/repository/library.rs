use std::path::PathBuf;

use crate::loaders::maven::MavenData;

pub struct SimpleLib {
    pub jar: PathBuf,
}

impl From<MavenData> for SimpleLib {
    fn from(value: MavenData) -> Self {
        Self { jar: value.path }
    }
}
