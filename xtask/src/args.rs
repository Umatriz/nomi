use clap::{Parser, Subcommand};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{config::Config, DynError};

#[derive(Debug, Parser)]
pub struct Xtask {
    /// Path to `Xtask.toml`
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Build the project with `--release` flag
    Build {
        /// Create a zip file with binary and moved files
        #[arg(long)]
        zip: bool,

        /// Move linked in `Xtask.toml` files to the build directory and include them in the zip
        #[arg(long)]
        move_files: bool,
    },
}

impl Xtask {
    pub fn build_release(&self, zip: bool, move_files: bool) -> Result<(), DynError> {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let status = Command::new(cargo)
            .current_dir(project_root())
            .args(["build", "--release"])
            .status()?;

        if !status.success() {
            Err("cargo build failed")?;
        }

        let target = project_root().join("dist/target/release/client.exe");

        let dist_dir = project_root().join("dist");
        let build_dir = dist_dir.join("build");
        std::fs::create_dir_all(&build_dir)?;
        std::fs::copy(target, &build_dir.join("client.exe"))?;

        if move_files {
            let path = Path::new("./Xtask.toml").to_path_buf();
            let config = Config::read(match self.config.as_ref() {
                Some(c) => c,
                None => &path,
            })?;
            let Some(folders) = config.move_folders else {
                return Err("You must specify `move_folders` field in `Xtask.toml`".into());
            };
            for (origin, new) in folders.iter() {
                match origin.is_file() || new.is_file() {
                    true => {
                        std::fs::copy(origin, &build_dir.join(new))?;
                    }
                    false => match origin.is_dir() || new.is_dir() {
                        true => copy_dir_all(origin, &build_dir.join(new))?,
                        false => return Err("Expected paths are not file/dir".into()),
                    },
                }
            }
        }

        if zip {
            crate::zip_dir::zip(
                build_dir.to_str().unwrap(),
                dist_dir.join("build.zip").to_str().unwrap(),
                crate::zip_dir::METHOD_STORED.unwrap(),
            )?;
        }

        Ok(())
    }
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
