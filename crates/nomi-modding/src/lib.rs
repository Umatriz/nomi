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
        reqwest::get(self.data.builder().build())
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
    base: String,
    data: Vec<String>,
}

impl Builder {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            data: Vec::new(),
        }
    }

    pub fn check_and_add_symbol(&mut self) {
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
        format!("{}{}", self.base, self.data.join(""))
    }
}

/// # Panics
/// Panics if the string is empty.
pub(crate) fn capitalize_first_letter(s: impl Into<String>) -> String {
    let mut chars = s.into().chars().map(String::from).collect::<Vec<String>>();
    chars[0] = chars[0].to_uppercase().to_string();
    chars.join("")
}

/// # Panics
/// Panics if the string is empty.
pub(crate) fn capitalize_first_letters_whitespace_splitted(s: impl Into<String>) -> String {
    let s: String = s.into();
    let iter = s.split_whitespace().map(capitalize_first_letter);

    itertools::intersperse(iter, " ".to_owned()).collect::<String>()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use categories::CategoriesData;
    use search::{Facets, InnerPart, Parts, ProjectType, SearchData};

    use super::*;

    #[test]
    fn capitalize_test() {
        assert_eq!("Ab", capitalize_first_letter("ab"));
        assert_eq!(
            "Ab Ba",
            capitalize_first_letters_whitespace_splitted("ab ba")
        );
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
}
