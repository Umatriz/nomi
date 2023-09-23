use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod toml;

#[async_trait(?Send)]
pub trait Config<T>
where
    T: Sized + Clone + for<'de> Deserialize<'de> + Serialize,
{
    async fn write(&self) -> anyhow::Result<()>;
    async fn read(&self) -> anyhow::Result<T>;
}
