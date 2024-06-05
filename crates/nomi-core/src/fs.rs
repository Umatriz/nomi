use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};
use tokio::io::AsyncWriteExt;

pub async fn write_toml_config<T>(data: &T, path: impl AsRef<Path>) -> anyhow::Result<()>
where
    T: Serialize + ?Sized,
{
    let path = path.as_ref();
    let body = toml::to_string_pretty(data)?;
    write_to_file(body.as_bytes(), path).await?;

    tracing::info!(
        "Config {} has been created successfully",
        path.to_string_lossy()
    );

    Ok(())
}

pub async fn read_toml_config<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned + ?Sized,
{
    let path = path.as_ref();

    let string = tokio::fs::read_to_string(&path).await?;
    let body: T = toml::from_str(&string)?;

    tracing::info!(
        "Config {} has been read successfully",
        path.to_string_lossy()
    );

    Ok(body)
}

pub fn read_toml_config_sync<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned + ?Sized,
{
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;
    runtime.block_on(read_toml_config::<T>(path))
}

pub fn write_toml_config_sync<T>(data: &T, path: impl AsRef<Path>) -> anyhow::Result<()>
where
    T: Serialize + ?Sized,
{
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;
    runtime.block_on(write_toml_config::<T>(data, path))
}

pub async fn read_json_config<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned + ?Sized,
{
    let s = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str::<T>(&s)?)
}

pub async fn write_json_config<T>(data: &T, path: impl AsRef<Path>) -> anyhow::Result<()>
where
    T: Serialize + ?Sized,
{
    let path = path.as_ref();
    let body = serde_json::to_string_pretty(data)?;

    write_to_file(body.as_bytes(), path).await?;

    tracing::info!("Config {} created successfully", path.to_string_lossy());

    Ok(())
}

pub async fn write_to_file(data: &[u8], path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    let mut file = tokio::fs::File::create(&path).await?;

    file.write_all(data).await?;

    Ok(())
}
