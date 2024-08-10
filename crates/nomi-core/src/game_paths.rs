use std::path::{Path, PathBuf};

use crate::{ASSETS_DIR, LIBRARIES_DIR, MINECRAFT_DIR};

#[derive(Debug, Clone)]
pub struct GamePaths {
    pub game: PathBuf,
    pub assets: PathBuf,
    pub profile: PathBuf,
    pub libraries: PathBuf,
}

impl GamePaths {
    pub fn from_instance_path(instance: impl AsRef<Path>, game_version: &str) -> Self {
        let path = instance.as_ref();

        Self {
            game: path.to_path_buf(),
            assets: ASSETS_DIR.into(),
            // Is this a good approach?
            profile: path.join("profiles").join(game_version),
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

    pub fn profile(&self) -> PathBuf {
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
}

impl Default for GamePaths {
    fn default() -> Self {
        Self {
            game: MINECRAFT_DIR.into(),
            assets: PathBuf::from(MINECRAFT_DIR).join("assets"),
            profile: PathBuf::from(MINECRAFT_DIR).join("versions").join("NOMI_DEFAULT"),
            libraries: PathBuf::from(MINECRAFT_DIR).join("libraries"),
        }
    }
}
