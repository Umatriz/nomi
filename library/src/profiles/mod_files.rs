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
    fn create_with_downloaded_by_short_code_data(short_code: &String) -> Result<Self> where Self: Sized;
}

#[async_trait]
pub trait DownloadedFromSiteModFile {
    fn url_to_file(&self) -> String;

    fn original_filename(&self) -> String;

    fn modfile_id(&self) -> String;

    fn filename_with_modfile_id(&self) -> String {
        format!("{}-{}", self.modfile_id(), self.original_filename())
    }

    async fn download_to_dir(&self, path_to_dir: PathBuf) -> Result<()> {
        let mut file = File::create(path_to_dir.join(self.original_filename()))?;
        let file_url = self.url_to_file();
        spawn_blocking(move || -> Result<(), reqwest::Error> {
            blocking::get(file_url)?.copy_to(&mut file)?;
            Ok(())
        })
        .await??;

        Ok(())
    }
    
    fn supports_loader(&self, loader: &Loader) -> Result<bool>;

    fn supports_version(&self, mc_version: &String) -> bool;

    fn string_versions(&self) -> Vec<String>;
}


impl DownloadedFromSiteModFile for ModrinthModFile {
    fn url_to_file(&self) -> String {
        self.file.url.clone()
    }

    fn original_filename(&self) -> String {
        self.file.filename.clone()
    }

    fn modfile_id(&self) -> String {
        format!("{}-{}-{}", self.project.id, self.version.id, self.file.filename)
    }

    fn supports_loader(&self, loader: &Loader) -> Result<bool> {
        Ok(self.version.loaders.contains(&loader.to_modrinth_loadersupport()?))
    }

    fn supports_version(&self, mc_version: &String) -> bool {
        // TODO: prevent errors caused by different versions formatting
        self.version.game_versions.contains(mc_version)
    }

    fn string_versions(&self) -> Vec<String> {
        self.version.game_versions.clone()
    }
}


impl ShortCodableModFile for ModrinthModFile {
    fn short_code(&self) -> Result<String> {
        // TODO: change slug to id
        let mod_name = escape_for_shortcode(
            &self.project.slug.clone().context("failed to get project slug")?
        );
        let file_name = escape_for_shortcode(&self.file.filename);

        Ok(format!("m:{mod_name}:{file_name}"))
    }

    fn create_with_downloaded_by_short_code_data(short_code: &String) -> Result<Self> where Self: Sized {
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


#[derive(Clone)]
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
