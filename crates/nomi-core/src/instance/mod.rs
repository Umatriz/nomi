use const_typed_builder::Builder;
use std::path::{Path, PathBuf};
use tracing::info;

pub mod launch;
pub mod profile;

use crate::{
    configs::profile::{VersionProfile, VersionProfileBuilder, VersionProfilesConfig},
    downloads::assets::AssetsDownload,
    loaders::{fabric::Fabric, vanilla::Vanilla},
    utils::state::{launcher_manifest_state_try_init, LAUNCHER_MANIFEST_STATE},
    version::download::DownloadVersion,
};

use self::launch::{LaunchInstance, LaunchInstanceBuilder, LaunchSettings};

#[derive(Default, Debug)]
pub struct Undefined;

#[derive(Debug, Builder)]
pub struct Instance {
    instance: Inner,
    pub version: String,
    pub libraries: PathBuf,
    pub version_path: PathBuf,
    pub assets: PathBuf,
    pub game: PathBuf,
    pub name: String,
}

#[derive(Debug)]
pub enum Inner {
    Vanilla(Box<Vanilla>),
    Fabric(Box<Fabric>),
}

impl Inner {
    pub async fn vanilla(version_id: impl Into<String>) -> anyhow::Result<Inner> {
        Ok(Inner::Vanilla(Box::new(Vanilla::new(version_id).await?)))
    }

    pub async fn fabric(
        game_version: impl Into<String>,
        loader_version: Option<impl Into<String>>,
    ) -> anyhow::Result<Inner> {
        Ok(Inner::Fabric(Box::new(
            Fabric::new(game_version, loader_version).await?,
        )))
    }
}

impl Instance {
    pub async fn download(&self) -> anyhow::Result<()> {
        match &self.instance {
            Inner::Vanilla(inner) => {
                inner.download(&self.version_path, &self.version).await?;
                inner.download_libraries(&self.libraries).await?;
                inner.create_json(&self.version_path).await?;
            }
            Inner::Fabric(inner) => {
                inner.download(&self.version_path, &self.version).await?;
                inner.download_libraries(&self.libraries).await?;
                inner.create_json(&self.version_path).await?;
            }
        }

        Ok(())
    }

    pub async fn assets(&self) -> anyhow::Result<AssetsInstance> {
        let manifest = LAUNCHER_MANIFEST_STATE
            .get_or_try_init(launcher_manifest_state_try_init)
            .await?;
        let version_manifest = manifest.get_version_manifest(&self.version).await?;

        AssetsInstanceBuilder::new(&self.version)
            .id(version_manifest.asset_index.id)
            .url(version_manifest.asset_index.url)
            .indexes(self.assets.join("indexes"))
            .objects(self.assets.join("objects"))
            .build()
            .await
    }

    pub fn launch_instance(&self, settings: LaunchSettings) -> LaunchInstance {
        let builder = LaunchInstanceBuilder::new().settings(settings);
        match &self.instance {
            Inner::Vanilla(_) => builder.build(),
            Inner::Fabric(inner) => builder.profile(&inner.profile).build(),
        }
    }

    pub fn into_profile(
        &self,
        profiles: &VersionProfilesConfig,
        version_type: String,
        is_downloaded: bool,
    ) -> VersionProfile {
        let builder = VersionProfileBuilder::new()
            .id(profiles.create_id())
            .name(self.name.clone())
            .assets(self.assets.clone())
            .game_dir(self.game.clone())
            .is_downloaded(is_downloaded)
            .libraries_dir(self.libraries.clone())
            .manifest_file(self.version_path.join(format!("{}.json", self.version)))
            .natives_dir(self.version_path.join("natives"))
            .version(self.version.clone())
            .version_jar_file(self.version_path.join(format!("{}.jar", self.version)))
            .version_type(version_type);

        match &self.instance {
            Inner::Fabric(fabric) => builder
                .profile_file(Some(
                    self.version_path
                        .join(format!("{}.json", fabric.profile.id)),
                ))
                .build(),
            Inner::Vanilla(_) => builder.build(),
        }
    }
}

pub struct AssetsInstance {
    inner: AssetsDownload,
    objects: PathBuf,
    indexes: PathBuf,
}

impl AssetsInstance {
    pub async fn download(&self) -> anyhow::Result<()> {
        self.inner.download_assets_chunked(&self.objects).await?;
        info!("Assets downloaded successfully");

        self.inner.get_assets_json(&self.indexes).await?;
        info!("Assets json created successfully");

        Ok(())
    }
}

#[derive(Default)]
pub struct AssetsInstanceBuilder<O, I, U, N> {
    version: String,
    objects: O,
    indexes: I,
    url: U,
    id: N,
}

impl AssetsInstanceBuilder<Undefined, Undefined, Undefined, Undefined> {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            ..Default::default()
        }
    }
}

impl AssetsInstanceBuilder<PathBuf, PathBuf, String, String> {
    pub async fn build(self) -> anyhow::Result<AssetsInstance> {
        let assets = AssetsDownload::new(self.url, self.id).await?;

        Ok(AssetsInstance {
            inner: assets,
            objects: self.objects,
            indexes: self.indexes,
        })
    }
}

impl<I, U, N> AssetsInstanceBuilder<Undefined, I, U, N> {
    pub fn objects(self, objects: impl AsRef<Path>) -> AssetsInstanceBuilder<PathBuf, I, U, N> {
        AssetsInstanceBuilder {
            version: self.version,
            objects: objects.as_ref().to_path_buf(),
            indexes: self.indexes,
            url: self.url,
            id: self.id,
        }
    }
}

impl<O, U, N> AssetsInstanceBuilder<O, Undefined, U, N> {
    pub fn indexes(self, indexes: impl AsRef<Path>) -> AssetsInstanceBuilder<O, PathBuf, U, N> {
        AssetsInstanceBuilder {
            version: self.version,
            objects: self.objects,
            indexes: indexes.as_ref().to_path_buf(),
            url: self.url,
            id: self.id,
        }
    }
}

impl<O, I, N> AssetsInstanceBuilder<O, I, Undefined, N> {
    pub fn url(self, url: impl Into<String>) -> AssetsInstanceBuilder<O, I, String, N> {
        AssetsInstanceBuilder {
            version: self.version,
            objects: self.objects,
            indexes: self.indexes,
            url: url.into(),
            id: self.id,
        }
    }
}

impl<O, I, U> AssetsInstanceBuilder<O, I, U, Undefined> {
    pub fn id(self, id: impl Into<String>) -> AssetsInstanceBuilder<O, I, U, String> {
        AssetsInstanceBuilder {
            version: self.version,
            objects: self.objects,
            indexes: self.indexes,
            url: self.url,
            id: id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build_test() {
        let _builder = InstanceBuilder::new()
            .version("1.18.2".into())
            .libraries("./minecraft/libraries".into())
            .version_path("./minecraft/instances/1.18.2".into())
            .instance(Inner::vanilla("1.18.2").await.unwrap())
            .assets("./minecraft/assets".into())
            .game("./minecraft".into())
            .name("1.18.2-minecraft".into())
            .build();
    }

    #[tokio::test]
    async fn assets_test() {
        let builder = InstanceBuilder::new()
            .version("1.18.2".into())
            .libraries("./minecraft/libraries".into())
            .version_path("./minecraft/instances/1.18.2".into())
            .instance(Inner::vanilla("1.18.2").await.unwrap())
            .assets("./minecraft/assets".into())
            .game("./minecraft".into())
            .name("1.18.2-minecraft".into())
            .build();

        builder.assets().await.unwrap().download().await.unwrap();
    }

    #[tokio::test]
    async fn fabric_test() {
        let subscriber = tracing_subscriber::fmt()
            .pretty()
            .with_max_level(tracing::Level::INFO)
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();

        let builder = InstanceBuilder::new()
            .version("1.18.2".into())
            .libraries("./minecraft/libraries".into())
            .version_path("./minecraft/instances/1.18.2".into())
            .instance(Inner::fabric("1.18.2", None::<String>).await.unwrap())
            .assets("./minecraft/assets".into())
            .game("./minecraft".into())
            .name("1.18.2-minecraft".into())
            .build();

        // builder.assets().await.unwrap().download().await.unwrap();

        builder.download().await.unwrap();
    }
}
