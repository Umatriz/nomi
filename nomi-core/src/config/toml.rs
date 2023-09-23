use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::Config;

pub struct TomlConfig<P>
where
    P: Sized + for<'de> Deserialize<'de> + serde::de::DeserializeOwned + Serialize,
{
    pub payload: Option<P>,
    pub path: PathBuf,
}

impl<P> TomlConfig<P>
where
    P: Sized + Clone + for<'de> Deserialize<'de> + Serialize,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(path: impl AsRef<Path>, payload: Option<P>) -> Self {
        Self {
            payload,
            path: path.as_ref().to_path_buf(),
        }
    }
}

#[async_trait(?Send)]
impl<T> Config<T> for TomlConfig<T>
where
    T: Sized + Clone + for<'de> Deserialize<'de> + Serialize,
{
    async fn write(&self) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(&self.payload.as_ref())?;

        if let Some(path) = self.path.parent() {
            tokio::fs::create_dir_all(path).await.map_err(|err| {
                error!(
                    "Error occurred during dirs creating\nPath: {}\nError: {}",
                    self.path.to_string_lossy(),
                    err
                );
                err
            })?;
        }

        match tokio::fs::write(&self.path, content).await {
            Ok(_) => {
                info!("Config {} updated", &self.path.to_string_lossy());
                Ok(())
            }
            Err(err) => {
                error!(
                    "Error occurred during config writing\nConfig path: {}\nError: {}",
                    &self.path.to_string_lossy(),
                    err
                );

                Err(err.into())
            }
        }
    }

    async fn read(&self) -> anyhow::Result<T> {
        let content = tokio::fs::read_to_string(&self.path).await.map_err(|err| {
            error!(
                "Error occurred during config reading\nConfig path: {}\nError: {}",
                self.path.to_string_lossy(),
                err
            );
            err
        })?;

        let data: T = toml::from_str(&content).map_err(|err| {
            error!(
                "Error occurred during config converting\nConfig path: {}\nError: {}",
                self.path.to_string_lossy(),
                err
            );
            err
        })?;

        Ok(data)
    }
}
