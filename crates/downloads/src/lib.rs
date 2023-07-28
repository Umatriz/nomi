use std::path::Path;

use anyhow::Context;
use reqwest::blocking;
use tokio::task::spawn_blocking;

pub(crate) mod launcher_manifest;
pub(crate) mod manifest;

pub(crate) mod assets;

pub mod jvm_dowload;
pub mod version;

pub(crate) async fn dowload_file<P: AsRef<Path>>(path: P, url: String) -> anyhow::Result<()> {
    let path = path.as_ref();

    if let Some(path) = path.parent() {
        std::fs::create_dir_all(path)?;
    }

    let mut file = std::fs::File::create(path).context("failed to create file")?;

    let _response = spawn_blocking(move || -> anyhow::Result<()> {
        blocking::get(&url)
            .map_err(|err| {
                log::error!("Error occurred during GET\nUrl: {}\nError: {}", &url, err);
                err
            })?
            .copy_to(&mut file)
            .map_err(|err| {
                log::error!("Error occurred during content copying\nError: {}", err);
                err
            })?;

        Ok(())
    })
    .await?;

    log::info!("Downloaded successfully {}", path.to_string_lossy());

    Ok(())
}
