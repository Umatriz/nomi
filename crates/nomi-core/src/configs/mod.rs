use std::{io::Write, path::Path};

use serde::{de::DeserializeOwned, Serialize};
use tokio::io::AsyncWriteExt;

pub mod profile;
pub mod user;
pub mod variables;

/// write data to a file
pub async fn write_toml_config<T: ?Sized>(data: &T, path: impl AsRef<Path>) -> anyhow::Result<()>
where
    T: Serialize,
{
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    let mut file = tokio::fs::File::create(&path).await?;

    let body = toml::to_string_pretty(data)?;

    file.write_all(body.as_bytes()).await?;

    tracing::info!("Config {} created successfully", path.to_string_lossy());

    Ok(())
}

/// read data from file
pub async fn read_toml_config<T: ?Sized>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();

    let s = tokio::fs::read_to_string(&path).await?;
    let body: T = toml::from_str(&s)?;

    tracing::info!("Config {} read successfully", path.to_string_lossy());

    Ok(body)
}

pub fn read_toml_config_sync<T: ?Sized>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();

    let s = std::fs::read_to_string(path)?;
    let body: T = toml::from_str(&s)?;

    tracing::info!("Config {} read successfully", path.to_string_lossy());

    Ok(body)
}

pub fn write_toml_config_sync<T: ?Sized>(data: &T, path: impl AsRef<Path>) -> anyhow::Result<()>
where
    T: Serialize,
{
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut file = std::fs::File::create(path)?;

    let body = toml::to_string_pretty(data)?;

    file.write_all(body.as_bytes())?;

    tracing::info!("Config {} created successfully", path.to_string_lossy());

    Ok(())
}
