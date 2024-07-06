use std::marker::PhantomData;

use search::{Facets, SearchData};
use serde::de::DeserializeOwned;

mod queries;
pub use queries::*;

pub struct Query<Data, T>
where
    Data: QueryBuilder<T>,
{
    data: Data,
    _marker: PhantomData<T>,
}

impl<Data, T> Query<Data, T>
where
    Data: QueryBuilder<T>,
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

pub trait QueryBuilder<T> {
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

#[tokio::test]
async fn feature() {
    let data = SearchData::builder()
        .facets(Facets::mods())
        .offset(5)
        .build();

    let query = Query::new(data);
    let data = query.query().await.unwrap();

    println!("{:#?}", data)
}
