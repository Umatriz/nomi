use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::repository::{simple_args::SimpleArgs, simple_lib::SimpleLib};

/// Read from json function
pub async fn read<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned + ?Sized,
{
    let s = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str::<T>(&s)?)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoaderProfile {
    pub name: String,
    pub main_class: String,
    pub args: SimpleArgs,
    pub libraries: Vec<SimpleLib>,
}
