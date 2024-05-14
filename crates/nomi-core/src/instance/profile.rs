use std::path::Path;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    repository::{simple_args::SimpleArgs, simple_lib::SimpleLib},
    utils::write_to_file,
};

pub async fn read_json<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned + ?Sized,
{
    let s = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str::<T>(&s)?)
}

pub async fn write_json<T: ?Sized>(data: &T, path: impl AsRef<Path>) -> anyhow::Result<()>
where
    T: Serialize,
{
    let path = path.as_ref();
    let body = serde_json::to_string_pretty(data)?;

    write_to_file(body.as_bytes(), path).await?;

    tracing::info!("Config {} created successfully", path.to_string_lossy());

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoaderProfile {
    pub name: String,
    pub main_class: String,
    pub args: SimpleArgs,
    pub libraries: Vec<SimpleLib>,
}
