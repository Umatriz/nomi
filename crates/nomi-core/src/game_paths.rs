use std::path::PathBuf;

use crate::MINECRAFT_DIR;

#[derive(Debug, Clone)]
pub struct GamePaths {
    pub game: PathBuf,
    pub assets: PathBuf,
    pub version: PathBuf,
    pub libraries: PathBuf,
}

impl Default for GamePaths {
    fn default() -> Self {
        Self {
            game: MINECRAFT_DIR.into(),
            assets: PathBuf::from(MINECRAFT_DIR).join("assets"),
            version: PathBuf::from(MINECRAFT_DIR).join("versions").join("NOMI_DEFAULT"),
            libraries: PathBuf::from(MINECRAFT_DIR).join("libraries"),
        }
    }
}
