use anyhow::{Context, Result};
use std::path::PathBuf;
use thiserror::Error;

pub struct GetPath;

impl GetPath {
    pub fn config() -> Result<PathBuf> {
        // TODO: Remove this .join()
        return Ok(std::env::current_dir()?.join("config.json"));
    }

    pub fn game() -> Result<PathBuf> {
        return Ok(std::env::current_dir()?.join("minecraft"));
    }

    pub fn get_java_bin() -> Result<PathBuf> {
        // TODO: looks like it \/ will not work on linux
        let _path = std::env::var("Path").context("failed to get `path` env var")?;
        let path_vec = _path.split(';').collect::<Vec<&str>>();
        let mut java_bin: Option<PathBuf> = None;
        for i in path_vec.iter() {
            if i.contains("java") {
                let pb = PathBuf::from(i).join("java.exe");
                match pb.exists() {
                    true => java_bin = Some(pb),
                    false => java_bin = None,
                }
            }
        }
        return java_bin.ok_or(GetPathError::CantFindJavaBin.into());
    }
}

#[derive(Error, Debug)]
pub enum GetPathError {
    #[error("can't find java executables")]
    CantFindJavaBin,
}
