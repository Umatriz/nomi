use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use std::io::Cursor;
use thiserror::Error;
use tokio::task::spawn_blocking;
use reqwest::blocking;
use zip_extract;
use sha256;

struct JavaInstaller;

const INSTALLER_URL: &str = "https://download.oracle.com/java/17/archive/jdk-17.0.7_windows-x64_bin.exe";
const PORTABLE_URL: &str = "https://download.oracle.com/java/17/archive/jdk-17.0.7_windows-x64_bin.zip";
// i hope they will not redesign site

const JDK_17_0_7_INSTALLER_SHA256: &str = "f41cfb7fd675f9f74b76217a2c0940b76f4676f053fddb62a464eacffa4a773b";
const JDK_17_0_7_PORTABLE_SHA256: &str = "c08fe96bc1af1b500ccbe7225475896d6859f66aa45e7c86e69906161b8cbaca";

impl JavaInstaller {
    async fn download(
        &self,
        temporary_dir_path: &PathBuf, 
        file_name: &str,
    ) -> anyhow::Result<()>{
        let mut file = File::create(temporary_dir_path.join(file_name))?;
        spawn_blocking(move || -> Result<(), reqwest::Error> {
            blocking::get(INSTALLER_URL)
                ?
                .copy_to(&mut file)
                ?;
            Ok(())
            }).await??;
        return Ok(());
    }

    fn check_hash(
        &self,
        path: &PathBuf,
        hash: &str,
    ) -> anyhow::Result<()> {
        if sha256::try_digest(path.as_path())? != hash {
            return Err(JavaInstallerError::HashDoesNotMatch.into());
        };
        return Ok(());
    }

    async fn install_java(
        &self,
        temporary_dir_path: &PathBuf,
    ) -> anyhow::Result<()> {
        let installer_file_name = "java_installer.exe";

        self.download(&temporary_dir_path, &installer_file_name).await?;
        self.check_hash(&temporary_dir_path.join(&installer_file_name), JDK_17_0_7_INSTALLER_SHA256)?;
        
        let path = {
            let joined_path = temporary_dir_path.join(&installer_file_name);
            joined_path.to_string_lossy().to_string()
        };
        
        Command::new(path)
            .arg("/s"); // silent, does not show gui

        return Ok(());
    }

    async fn prepare_portable_java(
        &self,
        temporary_dir_path: &PathBuf,
        java_dir_path: &Path,
    ) -> anyhow::Result<()> {
        let archive_filename = "java_portable.zip";
        self.download(&temporary_dir_path, archive_filename).await?;

        self.check_hash(&temporary_dir_path.join(archive_filename), JDK_17_0_7_PORTABLE_SHA256)?;

        zip_extract::extract(
            Cursor::new(
                std::fs::read(
                    temporary_dir_path.join(archive_filename)
                )?
            ), 
            java_dir_path, 
            true,
        )?;
        return Ok(());
    }
}

#[derive(Error, Debug)]
enum JavaInstallerError {
    #[error("Hash does not match")]
    HashDoesNotMatch
}
