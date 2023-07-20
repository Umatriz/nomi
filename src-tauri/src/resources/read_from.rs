use std::path::PathBuf;

pub trait ReadFrom {
    fn read_from_file(path: PathBuf) -> anyhow::Result<Self>
    where
        Self: Sized;
}
