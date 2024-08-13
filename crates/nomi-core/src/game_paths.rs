use std::path::{Path, PathBuf};

use crate::{
    instance::{Instance, InstanceProfileId},
    ASSETS_DIR, LIBRARIES_DIR,
};

#[derive(Debug, Clone)]
pub struct GamePaths {
    pub game: PathBuf,
    pub assets: PathBuf,
    pub profile: PathBuf,
    pub libraries: PathBuf,
}

impl GamePaths {
    pub fn from_id(id: InstanceProfileId) -> Self {
        Self::from_instance_path(Instance::path_from_id(id.instance()), id.profile())
    }

    pub fn from_instance_path(instance: impl AsRef<Path>, profile_id: usize) -> Self {
        let path = instance.as_ref();

        Self {
            game: path.to_path_buf(),
            assets: ASSETS_DIR.into(),
            // Is this a good approach?
            profile: path.join("profiles").join(format!("{profile_id}")),
            libraries: LIBRARIES_DIR.into(),
        }
    }

    pub fn make_absolute(self) -> anyhow::Result<Self> {
        let current_dir = std::env::current_dir()?;

        let make_path_absolute = |path: PathBuf| if path.is_absolute() { path } else { current_dir.join(path) };

        Ok(Self {
            game: make_path_absolute(self.game),
            assets: make_path_absolute(self.assets),
            profile: make_path_absolute(self.profile),
            libraries: make_path_absolute(self.libraries),
        })
    }

    pub fn profile_config(&self) -> PathBuf {
        self.profile.join("Profile.toml")
    }

    pub fn manifest_file(&self, game_version: &str) -> PathBuf {
        self.profile.join(format!("{game_version}.json"))
    }

    pub fn natives_dir(&self) -> PathBuf {
        self.profile.join("natives")
    }

    pub fn version_jar_file(&self, game_version: &str) -> PathBuf {
        self.profile.join(format!("{game_version}.jar"))
    }

    pub fn minecraft(game_version: &str) -> Self {
        const MINECRAFT_DIR: &str = "./minecraft";
        Self {
            game: MINECRAFT_DIR.into(),
            assets: PathBuf::from(MINECRAFT_DIR).join("assets"),
            profile: PathBuf::from(MINECRAFT_DIR).join("versions").join(game_version),
            libraries: PathBuf::from(MINECRAFT_DIR).join("libraries"),
        }
    }
}
