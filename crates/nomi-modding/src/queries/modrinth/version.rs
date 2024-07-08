//! Version

use std::ops::Deref;

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::project::{ProjectId, ProjectIdOrSlug};
use crate::{bool_as_str, format_list, QueryData};

pub type ProjectVersions = Vec<Version>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub name: String,
    pub version_number: String,
    pub changelog: String,
    pub dependencies: Vec<Dependency>,
    pub game_versions: Vec<String>,
    pub version_type: String,
    pub loaders: Vec<String>,
    pub featured: bool,
    pub status: String,
    pub requested_status: Option<String>,
    pub id: VersionId,
    pub project_id: ProjectId,
    pub author_id: String,
    pub date_published: String,
    pub downloads: i64,
    pub changelog_url: Option<String>,
    pub files: Vec<File>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionId(pub(crate) String);

impl Deref for VersionId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub version_id: Option<VersionId>,
    pub project_id: ProjectId,
    pub file_name: Option<String>,
    pub dependency_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hashes {
    pub sha512: String,
    pub sha1: String,
}

#[derive(Debug, TypedBuilder)]
pub struct ProjectVersionsData {
    #[builder(setter(into))]
    id_or_slug: ProjectIdOrSlug,
    #[builder(default, setter(strip_option))]
    loaders: Option<Vec<String>>,
    #[builder(default, setter(strip_option))]
    game_versions: Option<Vec<String>>,
    #[builder(default, setter(strip_option))]
    featured: Option<bool>,
}

impl QueryData<ProjectVersions> for ProjectVersionsData {
    fn builder(&self) -> crate::Builder {
        crate::Builder::new(format!(
            "https://api.modrinth.com/v2/project/{}/version",
            self.id_or_slug.value()
        ))
        .add_optional_parameter(
            "loaders",
            self.loaders.as_ref().map(|s| format_list(s.iter())),
        )
        .add_optional_parameter(
            "game_versions",
            self.game_versions.as_ref().map(|s| format_list(s.iter())),
        )
        .add_optional_parameter("featured", self.featured.map(bool_as_str))
    }
}

pub struct SingleVersionData {
    id: VersionId,
}

impl SingleVersionData {
    pub fn new(id: VersionId) -> Self {
        Self { id }
    }
}

impl QueryData<Version> for SingleVersionData {
    fn builder(&self) -> crate::Builder {
        crate::Builder::new(format!("https://api.modrinth.com/v2/version/{}", *self.id))
    }
}

#[derive(Default)]
pub struct MultipleVersionsData {
    ids: Vec<VersionId>,
}

impl MultipleVersionsData {
    pub fn new(ids: Vec<VersionId>) -> Self {
        Self { ids }
    }

    pub fn add_version(mut self, id: VersionId) -> Self {
        self.ids.push(id);
        self
    }
}

impl QueryData<ProjectVersions> for MultipleVersionsData {
    fn builder(&self) -> crate::Builder {
        crate::Builder::new("https://api.modrinth.com/v2/versions").add_parameter(
            "ids",
            format_list(self.ids.iter().map(<VersionId as Deref>::deref)),
        )
    }
}
