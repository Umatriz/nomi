use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use modrinth::types::{Project, ProjectVersion, VersionFile, LoaderSupport};
use reqwest::blocking;
use tokio::task::spawn_blocking;
use std::{path::PathBuf, fs::File};



pub struct ModrinthModFile {
    project: Project,
    version: ProjectVersion,
    file: VersionFile,
}

pub trait ShortCodableModFile {
    fn short_code(&self) -> Result<String>;
    fn from_short_code(short_code: String) -> Result<Self> where Self: Sized;
}

#[async_trait]
pub trait DownloadedFromSiteModFile {
    fn url_to_file(&self) -> String;

    fn filename(&self) -> String;

    async fn download_to_dir(&self, path_to_dir: PathBuf) -> Result<()> {
        let mut file = File::create(path_to_dir.join(self.filename()))?;
        let file_url = self.url_to_file();
        spawn_blocking(move || -> Result<(), reqwest::Error> {
            blocking::get(file_url)?.copy_to(&mut file)?;
            Ok(())
        })
        .await??;

        Ok(())
    }
    
    fn supports_loader(&self, loader: Loader) -> Result<bool>;
}


impl DownloadedFromSiteModFile for ModrinthModFile {
    fn url_to_file(&self) -> String {
        self.file.url.clone()
    }

    fn filename(&self) -> String {
        self.file.filename.clone()
    }

    fn supports_loader(&self, loader: Loader) -> Result<bool> {
        Ok(self.version.loaders.contains(&loader.to_modrinth_loadersupport()?))
    }
}


impl ShortCodableModFile for ModrinthModFile {
    fn short_code(&self) -> Result<String> {
        let mod_name = escape_for_shortcode(
            &self.project.slug.clone().context("failed to get project slug")?
        );
        let file_name = escape_for_shortcode(&self.file.filename);

        Ok(format!("m:{mod_name}:{file_name}"))
    }

    fn from_short_code(short_code: String) -> Result<Self> where Self: Sized {
        // TODO!!:
        todo!();    
    }
}


fn escape_for_shortcode(string: &str) -> String {
    string
    .replace('\\', "\\\\")
    .replace(':', "\\:")
    .replace(';', "\\;")
}


pub enum Loader {
    Forge,
    Fabric,
    Qulit,
    Other{name: String},
}


impl Loader {
    fn to_modrinth_loadersupport(&self) -> Result<LoaderSupport> {
        match self {
            Loader::Forge => Ok(LoaderSupport::Forge),
            Loader::Fabric => Ok(LoaderSupport::Fabric),
            _ => Err(anyhow!("unsupported by modrinth crate loader")),
        }
    }
}
