//! Project

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::QueryData;

use super::search::ProjectType;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub slug: ProjectSlug,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    pub body: String,
    pub status: String,
    pub requested_status: Option<String>,
    pub additional_categories: Vec<String>,
    pub issues_url: String,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Vec<DonationUrl>,
    pub project_type: ProjectType,
    pub downloads: i64,
    pub icon_url: String,
    pub color: i64,
    pub thread_id: String,
    pub monetization_status: String,
    pub id: ProjectId,
    pub team: String,
    pub body_url: Option<String>,
    pub moderator_message: Option<String>,
    pub published: String,
    pub updated: String,
    pub approved: String,
    pub queued: Option<String>,
    pub followers: i64,
    pub license: License,
    pub versions: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub gallery: Vec<Gallery>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
pub struct ProjectId(pub(crate) String);

impl Deref for ProjectId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSlug(pub(crate) String);

impl Deref for ProjectSlug {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum ProjectIdOrSlug {
    Slug(ProjectSlug),
    Id(ProjectId),
}

impl ProjectIdOrSlug {
    pub fn slug(slug: ProjectSlug) -> Self {
        Self::Slug(slug)
    }

    pub fn id(id: ProjectId) -> Self {
        Self::Id(id)
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Slug(slug) => slug,
            Self::Id(id) => id,
        }
    }
}

impl From<ProjectId> for ProjectIdOrSlug {
    fn from(value: ProjectId) -> Self {
        Self::Id(value)
    }
}

impl From<ProjectSlug> for ProjectIdOrSlug {
    fn from(value: ProjectSlug) -> Self {
        Self::Slug(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DonationUrl {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gallery {
    pub url: String,
    pub featured: bool,
    pub title: String,
    pub description: Option<String>,
    pub created: String,
    pub ordering: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

pub struct ProjectData {
    project_id_or_slug: ProjectIdOrSlug,
}

impl ProjectData {
    pub fn new(id_or_slug: impl Into<ProjectIdOrSlug>) -> Self {
        Self {
            project_id_or_slug: id_or_slug.into(),
        }
    }
}

impl QueryData<Project> for ProjectData {
    fn builder(&self) -> crate::Builder {
        crate::Builder::new(format!("https://api.modrinth.com/v2/project/{}", self.project_id_or_slug.value()))
    }
}
