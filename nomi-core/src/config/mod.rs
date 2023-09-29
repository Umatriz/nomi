use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod toml;

pub struct Config<State = Undefined> {
    data: State,
    path: PathBuf,
}

trait ConfigState {
    type Data: Serialize + for<'de> Deserialize<'de>;

    fn convert(&self) -> &Self::Data;
}

pub trait ConfigSetter {
    type Data: Serialize + for<'de> Deserialize<'de>;

    fn set(&mut self, data: Self::Data);
}

pub struct Undefined;

pub struct Borrowed<'a, T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    inner: &'a T,
}

impl<'a, D> ConfigState for Borrowed<'a, D>
where
    D: Serialize + for<'de> Deserialize<'de>,
{
    type Data = D;

    fn convert(&self) -> &Self::Data {
        self.inner
    }
}

impl<'a, D> ConfigSetter for Borrowed<'a, D>
where
    D: Serialize + for<'de> Deserialize<'de>,
    &'a D: for<'de> Deserialize<'de>,
{
    type Data = &'a D;

    fn set(&mut self, data: &'a D) {
        self.inner = data
    }
}

pub struct Owned<T>(T)
where
    T: Serialize + DeserializeOwned;

impl<D> ConfigState for Owned<D>
where
    D: Serialize + DeserializeOwned,
{
    type Data = D;

    fn convert(&self) -> &D {
        &self.0
    }
}

impl<D> ConfigSetter for Owned<D>
where
    D: Serialize + DeserializeOwned,
{
    type Data = D;

    fn set(&mut self, data: D) {
        self.0 = data
    }
}

impl Config<Undefined> {
    pub fn borrowed<D>(data: &D, path: impl AsRef<Path>) -> Config<Borrowed<'_, D>>
    where
        D: Serialize + for<'de> Deserialize<'de>,
    {
        Config {
            data: Borrowed { inner: data },
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn owned<D>(data: D, path: impl AsRef<Path>) -> Config<Owned<D>>
    where
        D: Serialize + DeserializeOwned,
    {
        Config {
            data: Owned(data),
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl<'a, T> Config<Borrowed<'a, T>>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn data(&self) -> &T {
        self.data.inner
    }
}

impl<T> Config<Owned<T>>
where
    T: Serialize + DeserializeOwned,
{
    pub fn data(&self) -> &T {
        &self.data.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_data() {
        let cfg = Config::borrowed(&[1, 2, 3], "./cfg.toml");

        let data = cfg.data();
    }
}
