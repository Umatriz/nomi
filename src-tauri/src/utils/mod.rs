use std::path::{Path, PathBuf};

use anyhow::Context;
use log::info;
use reqwest::blocking;
use tokio::task::spawn_blocking;

pub mod config;
pub mod logging;

pub async fn dowload<P: AsRef<Path>>(path: P, url: String) -> anyhow::Result<()> {
    let path = path.as_ref();
    let _ = std::fs::create_dir_all(path.parent().context("failed to get parent dir")?);

    let mut file = std::fs::File::create(path).context("failed to create file")?;

    let _response =
        spawn_blocking(move || blocking::get(url).unwrap().copy_to(&mut file).unwrap()).await;

    info!("Downloaded successfully {}", path.to_string_lossy());

    Ok(())
}

pub struct GetPath;

impl GetPath {
    pub fn config() -> PathBuf {
        std::env::current_dir().unwrap().join("config.json")
    }

    pub fn game() -> PathBuf {
        std::env::current_dir().unwrap().join("minecraft")
    }

    pub fn versions() -> PathBuf {
        Self::game().join("versions")
    }

    pub fn libraries() -> PathBuf {
        Self::game().join("libraries")
    }

    pub fn logs() -> PathBuf {
        std::env::current_dir().unwrap().join("logs")
    }

    pub fn java_bin() -> Option<PathBuf> {
        let _path = std::env::var("Path").unwrap();
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
        java_bin
    }
}
