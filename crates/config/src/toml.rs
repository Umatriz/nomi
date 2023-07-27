use std::{fs::create_dir_all, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::Config;

pub struct TomlConfig<'a, P>
where
    P: Sized + for<'de> Deserialize<'de> + serde::de::DeserializeOwned + Serialize,
{
    pub payload: Option<P>,
    pub path: &'a PathBuf,
}

pub struct TomlConfigBuilder<'a, P>
where
    P: Sized + Clone + for<'de> Deserialize<'de> + Serialize,
{
    payload: Option<P>,
    path: &'a PathBuf,
}

impl<P> TomlConfigBuilder<'_, P>
where
    P: Sized + Clone + for<'de> Deserialize<'de> + Serialize,
{
    pub fn payload(&mut self, payload: P) -> &mut Self {
        self.payload = Some(payload);
        self
    }

    pub fn build(&self) -> TomlConfig<P> {
        TomlConfig {
            payload: self.payload.clone(),
            path: self.path,
        }
    }
}

impl<P> TomlConfig<'_, P>
where
    P: Sized + Clone + for<'de> Deserialize<'de> + Serialize,
{
    #[allow(clippy::new_ret_no_self)]
    pub fn new(path: &PathBuf) -> TomlConfigBuilder<P> {
        TomlConfigBuilder {
            payload: None,
            path,
        }
    }
}

impl<T> Config<T> for TomlConfig<'_, T>
where
    T: Sized + Clone + for<'de> Deserialize<'de> + Serialize,
{
    fn write(&self) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(&self.payload.as_ref())?;

        if let Some(path) = self.path.parent() {
            create_dir_all(path).map_err(|err| {
                log::error!(
                    "Error occurred during dirs creating\nPath: {}\nError: {}",
                    self.path.to_string_lossy(),
                    err
                );
                err
            })?;
        }

        match std::fs::write(self.path, content) {
            Ok(_) => {
                log::info!("Config {} updated", &self.path.to_string_lossy());
                Ok(())
            }
            Err(err) => {
                log::error!(
                    "Error occurred during config writing\nConfig path: {}\nError: {}",
                    &self.path.to_string_lossy(),
                    err
                );

                Err(err.into())
            }
        }
    }

    fn read(&self) -> anyhow::Result<T> {
        let content = std::fs::read_to_string(self.path).map_err(|err| {
            log::error!(
                "Error occurred during config reading\nConfig path: {}\nError: {}",
                self.path.to_string_lossy(),
                err
            );
            err
        })?;

        let data: T = toml::from_str(&content).map_err(|err| {
            log::error!(
                "Error occurred during config converting\nConfig path: {}\nError: {}",
                self.path.to_string_lossy(),
                err
            );
            err
        })?;

        Ok(data)
    }
}
