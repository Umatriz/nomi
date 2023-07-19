use anyhow::Result;

use super::mod_files::{Loader, DownloadedFromSiteModFile};



struct ProfileSpec<'a> {
    mod_files: Vec<&'a dyn DownloadedFromSiteModFile>,
    loader: Loader,
    mc_version: String,
}

impl ProfileSpec<'_> {
    fn check_modfiles(&self) -> Result<Vec<ModFileError>> {
        let mut errors = vec![];
        
        for modfile in &self.mod_files {
            if !modfile.supports_version(&self.mc_version) {
                errors.extend(vec![
                    ModFileError::ModFileDoesNotSupportProfileVersion{
                        profile_version: self.mc_version.clone(),
                        modfile_versions: modfile.string_versions(),
                    }
                ])
            };

            if !modfile.supports_loader(&self.loader)? {
                errors.extend(vec![
                    ModFileError::ModFileDoesNotSupportLoader{
                        profile_loader: self.loader.clone(),
                    }
                ])
            }
        };

        return Ok(errors);
    }
}


#[derive(Clone)]
enum ModFileError {
    ModFileDoesNotSupportProfileVersion {
        profile_version: String,
        modfile_versions: Vec<String>,
    },
    ModFileDoesNotSupportLoader {
        profile_loader: Loader
    }
}
