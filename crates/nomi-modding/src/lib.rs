use std::marker::PhantomData;

use serde::de::DeserializeOwned;

mod queries;
pub use queries::*;

pub struct Query<Data, T>
where
    Data: QueryData<T>,
{
    data: Data,
    _marker: PhantomData<T>,
}

impl<Data, T> Query<Data, T>
where
    Data: QueryData<T>,
    T: DeserializeOwned,
{
    pub fn new(data: Data) -> Self {
        Self {
            data,
            _marker: PhantomData,
        }
    }

    pub async fn query(&self) -> Result<T, reqwest::Error> {
        reqwest::get(dbg!(self.data.builder().build()))
            .await?
            .json()
            .await
    }
}

pub trait QueryData<T> {
    /// Build the url.
    fn builder(&self) -> Builder;
}

pub struct Builder {
    base_url: String,
    data: Vec<String>,
}

impl Builder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            data: Vec::new(),
        }
    }

    fn check_and_add_symbol(&mut self) {
        if self.data.is_empty() {
            self.data.push("?".to_owned());
        } else {
            self.data.push("&".to_owned());
        }
    }

    pub fn add_optional_parameter(
        mut self,
        name: impl Into<String>,
        param: Option<impl Into<String>>,
    ) -> Self {
        if let Some(param) = param.map(Into::into) {
            self.check_and_add_symbol();
            self.data.push(format!("{}={}", name.into(), param));
        }
        self
    }

    pub fn add_parameter(mut self, name: impl Into<String>, param: impl Into<String>) -> Self {
        self.check_and_add_symbol();
        self.data.push(format!("{}={}", name.into(), param.into()));
        self
    }

    pub fn build(&self) -> String {
        format!("{}{}", self.base_url, self.data.join(""))
    }
}

/// # Panics
/// Panics if the string is empty.
pub fn capitalize_first_letter(s: impl Into<String>) -> String {
    let mut chars = s.into().chars().map(String::from).collect::<Vec<String>>();
    chars[0] = chars[0].to_uppercase().to_string();
    chars.join("")
}

/// # Panics
/// Panics if the string is empty.
pub fn capitalize_first_letters_whitespace_splitted(s: impl Into<String>) -> String {
    let s: String = s.into();
    let iter = s.split_whitespace().map(capitalize_first_letter);

    itertools::intersperse(iter, " ".to_owned()).collect::<String>()
}

pub(crate) fn format_list(value: impl Iterator<Item = impl Into<String>>) -> String {
    let iter = value.map(|s| format!("\"{}\"", s.into()));
    let s = itertools::intersperse(iter, ",".to_owned()).collect::<String>();
    format!("[{s}]")
}

/// Do not ask.
///
/// See implementation of `QueryData` for [`ProjectVersionsData`].
pub(crate) fn bool_as_str(val: bool) -> &'static str {
    ["false", "true"][val as usize]
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use modrinth::{
        categories::CategoriesData,
        dependencies::DependenciesData,
        project::{ProjectData, ProjectId, ProjectIdOrSlug},
        search::{Facets, InnerPart, Parts, ProjectType, Search, SearchData},
        version::{MultipleVersionsData, ProjectVersionsData, SingleVersionData},
    };

    use super::*;

    #[test]
    fn capitalize_test() {
        assert_eq!("Ab", capitalize_first_letter("ab"));
        assert_eq!(
            "Ab Ba",
            capitalize_first_letters_whitespace_splitted("ab ba")
        );
    }

    #[test]
    fn bool_as_str_test() {
        assert_eq!("true", bool_as_str(true));
        assert_eq!("false", bool_as_str(false));
    }

    async fn search_mods() -> Search {
        let data = SearchData::builder().facets(Facets::mods()).build();

        let query = Query::new(data);
        query.query().await.unwrap()
    }

    #[tokio::test]
    async fn search_test() {
        let data = SearchData::builder()
            .facets(Facets::new(
                Parts::new()
                    .add_part(InnerPart::new().add_category("atmosphere"))
                    .add_project_type(ProjectType::Shader),
            ))
            .build();

        let query = Query::new(data);
        let data = query.query().await.unwrap();

        println!("{:#?}", data)
    }

    #[tokio::test]
    async fn categories_test() {
        let query = Query::new(CategoriesData);
        let data = query.query().await.unwrap();

        println!("{:#?}", data)
    }

    #[tokio::test]
    async fn get_unique_categories_test() {
        let query = Query::new(CategoriesData);
        let data = query.query().await.unwrap();

        let data = data.get_unique_headers();

        println!("{:#?}", data)
    }

    #[tokio::test]
    async fn dependencies_test() {
        for project in search_mods().await.hits {
            let data = DependenciesData::new(ProjectIdOrSlug::id(project.project_id));
            let query = Query::new(data);
            let data = query.query().await.unwrap();
            println!("Success ({}): {:#?}", project.title, data.versions.first());
        }

        // Indium's ID: Orvt0mRa

        let data = DependenciesData::new(ProjectId("Orvt0mRa".into()));
        let query = Query::new(data);
        let data = query.query().await.unwrap();
        println!("Success (Indium): {:#?}", data.versions.first());
    }

    #[tokio::test]
    async fn project_test() {
        for project in search_mods().await.hits {
            let data = ProjectData::new(project.project_id);
            let query = Query::new(data);
            let data = query.query().await.unwrap();
            println!(
                "Success ({}): description - {}",
                project.title, data.description
            )
        }
    }

    #[tokio::test]
    async fn versions_test() {
        for project in search_mods().await.hits {
            let data = ProjectVersionsData::builder()
                .id_or_slug(project.slug)
                .loaders(["fabric"].map(String::from).to_vec())
                .game_versions(["1.19.2"].map(String::from).to_vec())
                .build();

            let query = Query::new(data);
            let versions = query.query().await.unwrap();

            for version in versions.iter().filter(|v| v.featured) {
                println!("Success ({}): {}", &project.title, version.name)
            }

            for version in &versions[..(versions.len() / 2)] {
                let data = SingleVersionData::new(version.id.clone());
                let query = Query::new(data);
                if let Ok(data) = query.query().await.inspect_err(|e| eprintln!("{:#?}", e)) {
                    println!("Success (single): {}", data.name);
                };
            }

            let data = MultipleVersionsData::new(
                versions[..(versions.len() / 2)]
                    .iter()
                    .map(|v| &v.id)
                    .cloned()
                    .collect_vec(),
            );
            let query = Query::new(data);
            let data = query.query().await.unwrap();

            for version in data {
                println!("Success (multiple): {}", version.name);
            }
        }
    }
}
