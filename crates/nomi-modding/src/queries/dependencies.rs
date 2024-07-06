//! Project's dependencies

use serde::{Deserialize, Serialize};

use super::{project::Project, version::Version};

#[derive(Serialize, Deserialize)]
pub struct Dependencies {
    pub projects: Vec<Project>,
    pub versions: Vec<Version>,
}
