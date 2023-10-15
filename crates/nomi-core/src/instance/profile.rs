use std::path::Path;

use serde::de::DeserializeOwned;

use crate::repository::{simple_args::SimpleArgs, simple_lib::SimpleLib};

/// Trait that must be implemented for every loader profile
pub trait Profile {
    fn name(&self) -> String;
    fn main_class(&self) -> String;
    fn arguments(&self) -> SimpleArgs;
    fn libraries(&self) -> Vec<SimpleLib>;
}

/// Read from json function
pub async fn read<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned + ?Sized,
{
    let s = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str::<T>(&s)?)
}
