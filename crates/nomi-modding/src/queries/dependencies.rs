//! Project's dependencies

use serde::{Deserialize, Serialize};

use crate::{project::ProjectId, Builder, QueryData};

use super::{project::Project, version::Version};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependencies {
    pub projects: Vec<Project>,
    pub versions: Vec<Version>,
}

pub struct DependenciesData {
    project_id: ProjectId,
}

impl DependenciesData {
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

impl QueryData<Dependencies> for DependenciesData {
    fn builder(&self) -> crate::Builder {
        Builder::new(format!(
            "https://api.modrinth.com/v2/project/{}/dependencies",
            *self.project_id
        ))
    }
}
