use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::maven_data::{MavenArtifact, MavenData};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleLib {
    pub artifact: MavenArtifact,
    pub jar: PathBuf,
}

impl From<MavenArtifact> for SimpleLib {
    fn from(value: MavenArtifact) -> Self {
        let maven_data = MavenData::from_artifact_data(&value);
        Self {
            jar: maven_data.path,
            artifact: value,
        }
    }
}
