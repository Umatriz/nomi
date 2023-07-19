use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error["This version does not exist."]]
    LauncherManifestVersionError,
}
