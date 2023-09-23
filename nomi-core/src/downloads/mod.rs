pub mod assets;
#[cfg(target_os = "windows")]
pub mod jvm_dowload;
pub mod utils;
pub mod version;

use std::path::Path;

use futures_util::stream::StreamExt;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error};

pub const LAUNCHER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

/*
    TODO: Improve downloading speed
    Step 1 would be to run the downloads concurrently
        For step 1 you can use a tokio JoinSet. In your loop, you spawn the download into the JoinSet.
        After the loop you will await all entries in the JoinSet
    Step 2 would be to switch to tokio::fs instead of std::fs
*/
pub(crate) async fn download_file<P: AsRef<Path>>(
    path: P,
    url: impl Into<String>,
) -> anyhow::Result<()> {
    let path = path.as_ref();

    if let Some(path) = path.parent() {
        tokio::fs::create_dir_all(path).await?;
    }

    let client = Client::new();
    let res = client.get(&url.into()).send().await?;

    let mut file = tokio::fs::File::create(path).await.map_err(|err| {
        error!(
            "Error occurred during file creating\nPath: {}\nError: {}",
            path.to_string_lossy(),
            err
        );
        err
    })?;

    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|err| {
            error!("Error occurred during file downloading\nError: {}", err);
            err
        })?;

        file.write_all(&chunk).await.map_err(|err| {
            error!("Error occurred during writing to file\nError: {}", err);
            err
        })?;
    }

    debug!("Downloaded successfully {}", path.to_string_lossy());

    Ok(())
}
