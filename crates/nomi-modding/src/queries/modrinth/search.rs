//! Search for projects

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{
    project::{ProjectId, ProjectSlug},
    version::VersionId,
};
use crate::{Builder, QueryData};

#[derive(Debug, Serialize, Deserialize)]
pub struct Search {
    pub hits: Vec<Hit>,
    pub offset: i64,
    pub limit: i64,
    pub total_hits: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hit {
    pub slug: ProjectSlug,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    pub project_type: ProjectType,
    pub downloads: i64,
    pub icon_url: String,
    pub color: Option<i64>,
    pub thread_id: Option<String>,
    pub monetization_status: Option<String>,
    pub project_id: ProjectId,
    pub author: String,
    pub display_categories: Vec<String>,
    pub versions: Vec<VersionId>,
    pub follows: i64,
    pub date_created: String,
    pub date_modified: String,
    pub latest_version: String,
    pub license: String,
    pub gallery: Vec<String>,
    pub featured_gallery: Option<String>,
}

#[derive(TypedBuilder)]
pub struct SearchData {
    #[builder(default, setter(strip_option, into))]
    query: Option<String>,
    #[builder(default, setter(strip_option))]
    facets: Option<Facets>,
    #[builder(default, setter(strip_option))]
    index: Option<Index>,
    #[builder(default, setter(strip_option))]
    offset: Option<u32>,

    /// Must be in the range 0..100
    #[builder(default, setter(strip_option))]
    limit: Option<u8>,
}

impl SearchData {
    pub fn set_query(&mut self, query: Option<String>) {
        self.query = query;
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Facets {
    parts: Parts,
}

impl Facets {
    pub fn new(parts: Parts) -> Self {
        Self { parts }
    }

    pub fn from_project_type(project_type: ProjectType) -> Self {
        Self {
            parts: Parts::from_project_type(project_type),
        }
    }

    pub fn parts_mut(&mut self) -> &mut Parts {
        &mut self.parts
    }

    pub fn build(&self) -> String {
        self.parts.build()
    }
}

/// Each [`InnerPart`] represents a statement that will be joined using `AND` operation.
/// Eg: `inner_part_1 AND inner_part_2` means that statement from `inner_part_1` and `inner_part_2`
/// must be satisfied.
///
/// Where items in [`InnerPart`] will be joined using `OR` operation. See [`InnerPart`]
/// for details.
#[derive(Default, Clone, PartialEq, Eq)]
pub struct Parts(Vec<InnerPart>);

impl Parts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_project_type(project_type: ProjectType) -> Self {
        let parts = Parts::new().add_project_type(project_type);
        InnerPart::from_project_type(project_type).map_or(parts.clone(), |inner| parts.part(inner))
    }

    pub fn part(mut self, part: InnerPart) -> Self {
        self.0.push(part);
        self
    }

    pub fn add_part(&mut self, part: InnerPart) -> &mut Self {
        self.0.push(part);
        self
    }

    pub fn add_project_type(mut self, project_type: ProjectType) -> Self {
        let inner = InnerPart::new().add_project_type(project_type);
        self.0.push(inner);
        self
    }

    pub fn build(&self) -> String {
        let iter = self.0.iter().map(|i| i.build());
        let string = itertools::intersperse(iter, ",".to_owned()).collect::<String>();
        format!("[{string}]")
    }
}

/// All items inside will be joined using OR operation.
///
/// Eg: `["categories:fabric", "categories:forge"]` mean that mod will be supported either by Fabric
/// or by Forge.
#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct InnerPart(Vec<String>);

impl InnerPart {
    pub fn new() -> Self {
        Self::default()
    }

    /// It must satisfy required format.
    ///
    /// Call [`Self::format_category`] and [`Self::format_version`] to format your data.
    ///
    /// Eg: category must be formatted in the form of `category:{CATEGORY_HERE}`.
    pub fn from_vec(vec: Vec<String>) -> Self {
        Self(vec)
    }

    pub fn from_project_type(project_type: ProjectType) -> Option<Self> {
        match project_type {
            ProjectType::Mod => Some(
                InnerPart::new()
                    .add_category("forge")
                    .add_category("fabric")
                    .add_category("quilt")
                    .add_category("liteloader")
                    .add_category("modloader")
                    .add_category("rift")
                    .add_category("neoforge"),
            ),
            ProjectType::Plugin => Some(
                InnerPart::new()
                    .add_category("bukkit")
                    .add_category("spigot")
                    .add_category("paper")
                    .add_category("purpur")
                    .add_category("sponge")
                    .add_category("bungeecord")
                    .add_category("waterfall")
                    .add_category("velocity")
                    .add_category("folia"),
            ),
            ProjectType::DataPack => Some(InnerPart::new().add_category("datapack")),
            _ => None,
        }
    }

    pub fn add_project_type(mut self, project_type: ProjectType) -> Self {
        self.0.push(project_type.as_facet());
        self
    }

    pub fn format_category(category: impl Into<String>) -> String {
        format!("categories:{}", category.into())
    }

    pub fn format_version(version: impl Into<String>) -> String {
        format!("versions:{}", version.into())
    }

    pub fn add_category(mut self, category: impl Into<String>) -> Self {
        self.0.push(Self::format_category(category));
        self
    }

    pub fn add_version(mut self, version: impl Into<String>) -> Self {
        self.0.push(Self::format_version(version));
        self
    }

    pub fn add_client_side(mut self) -> Self {
        self.0.push("client_side".to_owned());
        self
    }

    pub fn add_server_side(mut self) -> Self {
        self.0.push("server_side".to_owned());
        self
    }

    pub fn add_open_source(mut self) -> Self {
        self.0.push("open_source".to_owned());
        self
    }

    pub fn build(&self) -> String {
        let iter = self.0.iter().map(|s| format!("\"{}\"", s));
        let string = itertools::intersperse(iter, ",".to_owned()).collect::<String>();
        format!("[{string}]")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    #[default]
    Mod,
    Modpack,
    ResourcePack,
    Shader,
    Plugin,
    DataPack,
}

impl ProjectType {
    pub fn iter() -> impl Iterator<Item = ProjectType> {
        use ProjectType::*;
        [Mod, Modpack, ResourcePack, Shader, Plugin, DataPack].into_iter()
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Mod => "mod",
            Self::Modpack => "modpack",
            Self::ResourcePack => "resourcepack",
            Self::Shader => "shader",
            Self::Plugin => "plugin",
            Self::DataPack => "datapack",
        }
    }

    pub fn as_facet(&self) -> String {
        format!("project_type:{}", self.as_str())
    }
}

impl QueryData<Search> for SearchData {
    fn builder(&self) -> Builder {
        Builder::new("https://api.modrinth.com/v2/search")
            .add_optional_parameter("query", self.query.as_ref())
            .add_optional_parameter("facets", self.facets.as_ref().map(|f| f.build()))
            .add_optional_parameter("index", self.index.as_ref().map(|i| i.as_str()))
            .add_optional_parameter("offset", self.offset.map(|o| format!("{o}")))
            .add_optional_parameter("limit", self.limit.map(|l| format!("{l}")))
    }
}

#[derive(Default, Clone, Copy)]
pub enum Index {
    #[default]
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}

impl From<Index> for String {
    fn from(value: Index) -> Self {
        value.as_str().to_owned()
    }
}

impl Index {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Relevance => "relevance",
            Self::Downloads => "downloads",
            Self::Follows => "follows",
            Self::Newest => "newest",
            Self::Updated => "updated",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_test() {
        let s = Facets::from_project_type(ProjectType::Mod).parts.build();
        println!("{s}")
    }
}
