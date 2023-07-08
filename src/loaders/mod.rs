use async_trait::async_trait;

pub mod fabric;
pub mod fabric_meta;

pub mod maven;

pub const QUILT_META: &str = "https://meta.quiltmc.org/";
pub const QUILT_MAVEN: &str = "https://maven.quiltmc.org/";

pub const FABRIC_META: &str = "https://meta.fabricmc.net/";
pub const FABRIC_MAVEN: &str = "https://maven.fabricmc.net/";

#[async_trait]
pub trait Loader {
    async fn meta_struct<T>() -> T;

    async fn download() -> anyhow::Result<()>;
}
