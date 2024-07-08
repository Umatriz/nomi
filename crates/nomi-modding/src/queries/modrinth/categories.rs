//! Get a list of categories.

use std::{collections::HashSet, ops::Deref};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::QueryData;

use super::search::ProjectType;

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Categories(Vec<Category>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub icon: String,
    pub name: String,
    pub project_type: ProjectType,
    pub header: String,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct Header(String);

impl Deref for Header {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Categories {
    pub fn get_all_categories(&self) -> &[Category] {
        &self.0
    }

    pub fn get_unique_headers_with_project_type(&self) -> Vec<(Header, ProjectType)> {
        self.0
            .iter()
            .map(|c| (Header(c.header.clone()), c.project_type))
            .sorted()
            .dedup()
            .collect_vec()
    }

    pub fn get_unique_headers_sorted_for_project_type(
        &self,
        project_type: ProjectType,
    ) -> Vec<Header> {
        self.0
            .iter()
            .filter(|c| c.project_type == project_type)
            .map(|c| c.header.clone())
            .sorted()
            .dedup()
            .map(Header)
            .collect_vec()
    }

    pub fn get_unique_headers(&self) -> HashSet<Header> {
        self.0
            .iter()
            .map(|c| c.header.clone())
            .map(Header)
            .collect::<HashSet<_>>()
    }

    pub fn filter_by_header_and_project_type(
        &self,
        header: Header,
        project_type: ProjectType,
    ) -> Vec<&Category> {
        self.0
            .iter()
            .filter(|c| c.project_type == project_type && c.header == *header)
            .collect_vec()
    }
}

/// There's no data to be passed.
pub struct CategoriesData;

impl QueryData<Categories> for CategoriesData {
    fn builder(&self) -> crate::Builder {
        crate::Builder::new("https://api.modrinth.com/v2/tag/category")
    }
}
