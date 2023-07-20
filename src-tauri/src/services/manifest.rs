use std::path::PathBuf;

use crate::{data::manifest::Manifest, resources::read_from::ReadFrom};

impl ReadFrom for Manifest {
    fn read_from_file(path: PathBuf) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path)?;

        let content: Self = serde_json::from_reader(file)?;

        Ok(content)
    }
}
