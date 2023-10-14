use std::path::{Path, PathBuf};
use tracing::info;

pub mod launch;

use crate::{
    downloads::assets::AssetsDownload,
    loaders::{fabric::Fabric, vanilla::Vanilla},
    utils::state::LAUNCHER_MANIFEST_STATE,
    version::download::DownloadVersion,
};

#[derive(Default, Debug)]
pub struct Undefined;

#[derive(Debug)]
pub struct Instance {
    inner: Inner,
    version: String,
    game: PathBuf,
    libraries: PathBuf,
    version_path: PathBuf,
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
        match &self.inner {
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

    pub async fn assets(
        &self,
        version: impl Into<String>,
    ) -> anyhow::Result<AssetsInstanceBuilder<Undefined, Undefined, String, String>> {
        let version = version.into();

        let manifest = LAUNCHER_MANIFEST_STATE.get_or_try_init().await?;
        let version_manifest = manifest.get_version_manifest(&version).await?;

        Ok(AssetsInstanceBuilder::new(version)
            .id(version_manifest.asset_index.id)
            .url(version_manifest.asset_index.url))
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

#[derive(Debug, Default)]
pub struct InstanceBuilder<I, N, G, L, V> {
    inner: I,
    version: N,
    game: G,
    libraries: L,
    version_path: V,
}

impl InstanceBuilder<Undefined, Undefined, Undefined, Undefined, Undefined> {
    pub fn new() -> Self {
        InstanceBuilder::default()
    }
}

impl InstanceBuilder<Inner, String, PathBuf, PathBuf, PathBuf> {
    pub fn build(self) -> Instance {
        Instance {
            inner: self.inner,
            version: self.version,
            game: self.game,
            libraries: self.libraries,
            version_path: self.version_path,
        }
    }
}

impl<G, N, L, V> InstanceBuilder<Undefined, N, G, L, V> {
    pub fn instance(self, inner: Inner) -> InstanceBuilder<Inner, N, G, L, V> {
        InstanceBuilder {
            inner,
            version: self.version,
            game: self.game,
            libraries: self.libraries,
            version_path: self.version_path,
        }
    }

    pub async fn vanilla(
        self,
        version_id: impl Into<String>,
    ) -> anyhow::Result<InstanceBuilder<Inner, N, G, L, V>> {
        let inner = Inner::vanilla(version_id).await?;
        Ok(InstanceBuilder {
            inner,
            version: self.version,
            game: self.game,
            libraries: self.libraries,
            version_path: self.version_path,
        })
    }

    pub async fn fabric(
        self,
        game_version: impl Into<String>,
        loader_version: Option<impl Into<String>>,
    ) -> anyhow::Result<InstanceBuilder<Inner, N, G, L, V>> {
        let inner = Inner::fabric(game_version, loader_version).await?;
        Ok(InstanceBuilder {
            inner,
            version: self.version,
            game: self.game,
            libraries: self.libraries,
            version_path: self.version_path,
        })
    }
}

impl<I, N, L, V> InstanceBuilder<I, N, Undefined, L, V> {
    pub fn game(self, game: impl AsRef<Path>) -> InstanceBuilder<I, N, PathBuf, L, V> {
        InstanceBuilder {
            inner: self.inner,
            version: self.version,
            game: game.as_ref().to_path_buf(),
            libraries: self.libraries,
            version_path: self.version_path,
        }
    }
}

impl<I, N, G, V> InstanceBuilder<I, N, G, Undefined, V> {
    pub fn libraries(self, libraries: impl AsRef<Path>) -> InstanceBuilder<I, N, G, PathBuf, V> {
        InstanceBuilder {
            inner: self.inner,
            version: self.version,
            game: self.game,
            libraries: libraries.as_ref().to_path_buf(),
            version_path: self.version_path,
        }
    }
}

impl<I, N, G, L> InstanceBuilder<I, N, G, L, Undefined> {
    pub fn version_path(
        self,
        version_path: impl AsRef<Path>,
    ) -> InstanceBuilder<I, N, G, L, PathBuf> {
        InstanceBuilder {
            inner: self.inner,
            version: self.version,
            game: self.game,
            libraries: self.libraries,
            version_path: version_path.as_ref().to_path_buf(),
        }
    }
}

impl<I, G, L, V> InstanceBuilder<I, Undefined, G, L, V> {
    pub fn version(self, version: impl Into<String>) -> InstanceBuilder<I, String, G, L, V> {
        InstanceBuilder {
            inner: self.inner,
            version: version.into(),
            game: self.game,
            libraries: self.libraries,
            version_path: self.version_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build_test() {
        let _builder = InstanceBuilder::new()
            .version("1.18.2")
            .game("./minecraft")
            .libraries("./minecraft/libraries")
            .version_path("./minecraft/instances/1.18.2")
            .vanilla("1.18.2")
            .await
            .unwrap()
            .build();
    }

    #[tokio::test]
    async fn assets_test() {
        let builder = InstanceBuilder::new()
            .version("1.18.2")
            .game("./minecraft")
            .libraries("./minecraft/libraries")
            .version_path("./minecraft/instances/1.18.2")
            .vanilla("1.18.2")
            .await
            .unwrap()
            .build();

        builder
            .assets("1.18.2")
            .await
            .unwrap()
            .indexes("./minecraft/assets/indexes")
            .objects("./minecraft/assets/objects")
            .build()
            .await
            .unwrap()
            .download()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn fabric_test() {
        let subscriber = tracing_subscriber::fmt()
            .pretty()
            .with_max_level(tracing::Level::INFO)
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();

        let builder = InstanceBuilder::new()
            .version("1.18.2")
            .game("./minecraft")
            .libraries("./minecraft/libraries")
            .version_path("./minecraft/instances/1.18.2")
            .fabric("1.18.2", None::<String>)
            .await
            .unwrap()
            .build();

        // builder
        //     .assets("1.18.2")
        //     .await
        //     .unwrap()
        //     .indexes("./minecraft/assets/indexes")
        //     .objects("./minecraft/assets/objects")
        //     .build()
        //     .await
        //     .unwrap()
        //     .download()
        //     .await
        //     .unwrap();

        builder.download().await.unwrap();
    }
}
