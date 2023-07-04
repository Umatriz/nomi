use std::path::{Path, PathBuf};
use std::fs::File;
use std::process::Command;
use std::io::Cursor;
use tokio::task::spawn_blocking;
use reqwest::blocking;
use zip_extract;
use sha256;



struct JavaInstaller {}

const installer_url: &str = "https://download.oracle.com/java/17/archive/jdk-17.0.7_windows-x64_bin.exe";
const portable_url: &str = "https://download.oracle.com/java/17/archive/jdk-17.0.7_windows-x64_bin.zip";
// i hope they will not redesign site

const jdk_17_0_7_installer_sha256: &str = "f41cfb7fd675f9f74b76217a2c0940b76f4676f053fddb62a464eacffa4a773b";
const jdk_17_0_7_portable_sha256: &str = "c08fe96bc1af1b500ccbe7225475896d6859f66aa45e7c86e69906161b8cbaca";

impl JavaInstaller {
    async fn download_shit(
        &self,
        temporary_dir_path: &PathBuf, 
        file_name: &str,
    ) -> Result<(), std::io::Error>{
        let mut file = File::create(temporary_dir_path.join(file_name))?;
        spawn_blocking(move || { // TODO: remove expects
            blocking::get(installer_url)
                .expect("Failed to get java shit from site")
                .copy_to(&mut file)
                .expect("Failed to copy java shit into a file")
            }).await;
        return Ok(());
    }

    fn check_hash(
        &self,
        shit_path: &PathBuf,
        shit_hash: &str,
    ) -> Result<(), JavaInstallerError> {
        if sha256::try_digest(shit_path.as_path()).map_err(|x| JavaInstallerError::Sha256Error(x))? != shit_hash {
            return Err(JavaInstallerError::HashDoesNotMatch);
        };
        return Ok(());
    }

    async fn install_java(
        &self,
        temporary_dir_path: &PathBuf,
    ) -> Result<(), JavaInstallerError> {
        let installer_file_name = "java_installer.exe";

        self.download_shit(&temporary_dir_path, &installer_file_name).await;
        self.check_hash(&temporary_dir_path.join(&installer_file_name), jdk_17_0_7_installer_sha256);
        
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
    ) -> Result<(), JavaInstallerError> {
        let archive_filename = "java_portable.zip";
        self.download_shit(&temporary_dir_path, archive_filename).await;

        self.check_hash(&temporary_dir_path.join(archive_filename), jdk_17_0_7_portable_sha256)?;

        zip_extract::extract(
            Cursor::new(
                std::fs::read(
                    temporary_dir_path.join(archive_filename)
                ).map_err(|x| JavaInstallerError::FsError())?
            ), 
            java_dir_path, 
            true,
        ).map_err(|x| JavaInstallerError::ZipExtractionError(x))?;
        return Ok(());
    }
}

enum JavaInstallerError<'a> {
    PathToStrConvertationError,
    HashDoesNotMatch,
    HashingError,
    Sha256Error(<&'a std::path::Path as sha256::TrySha256Digest>::Error),
    ZipExtractionError(zip_extract::ZipExtractError),
    FsError(),
}
