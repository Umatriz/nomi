use std::{
    path::Path,
    process::{Child, Command},
};

/// # Panics
/// Panics if `dir` is not a directory.
pub fn open_directory_native(dir: impl AsRef<Path>) -> anyhow::Result<Child> {
    let dir = dir.as_ref();
    assert!(dir.is_dir());

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(dir).spawn().map_err(Into::into)
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(dir.as_ref()).spawn().map_err(Into::into)
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(dir.as_ref()).spawn().map_err(Into::into)
    }
}
