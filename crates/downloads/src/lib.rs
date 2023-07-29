use std::io::Write;
use std::path::Path;

use futures_util::stream::StreamExt;
use reqwest::{blocking, Client};
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

    let client = Client::new();
    let res = client.get(url).send().await?;

    let mut file = std::fs::File::create(path).map_err(|err| {
        log::error!(
            "Error occurred during file creating\nPath: {}\nError: {}",
            path.to_string_lossy(),
            err
        );
        err
    })?;

    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|err| {
            log::error!("Error occurred during file downloading\nError: {}", err);
            err
        })?;

        file.write_all(&chunk).map_err(|err| {
            log::error!("Error occurred during writing to file\nError: {}", err);
            err
        })?;
    }

    log::info!("Downloaded successfully {}", path.to_string_lossy());

    Ok(())
}
