//! Project's dependencies

use serde::{Deserialize, Serialize};

use crate::{Builder, QueryData};

use super::{
    project::{Project, ProjectIdOrSlug},
    version::Version,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependencies {
    pub projects: Vec<Project>,
    pub versions: Vec<Version>,
}

pub struct DependenciesData {
    project_id_or_slug: ProjectIdOrSlug,
}

impl DependenciesData {
    pub fn new(id_or_slug: impl Into<ProjectIdOrSlug>) -> Self {
        Self {
            project_id_or_slug: id_or_slug.into(),
        }
    }
}

impl QueryData<Dependencies> for DependenciesData {
    fn builder(&self) -> crate::Builder {
        Builder::new(format!(
            "https://api.modrinth.com/v2/project/{}/dependencies",
            self.project_id_or_slug.value()
        ))
    }
}
