use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct GamePaths {
    pub game: PathBuf,
    pub assets: PathBuf,
    pub version: PathBuf,
    pub libraries: PathBuf,
}
