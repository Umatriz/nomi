use serde::{Deserialize, Serialize};

pub mod toml;

pub trait Config<T>
where
    T: Sized + Clone + for<'de> Deserialize<'de> + Serialize,
{
    fn write(&self) -> anyhow::Result<()>;
    fn read(&self) -> anyhow::Result<T>;
}
