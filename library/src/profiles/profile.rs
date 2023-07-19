use std::{path::{PathBuf, Path}, fs::File};

use anyhow::Result;

use super::profile_spec::{ProfileSpec, ModFileError};

pub struct Profile<'a> {
    path: PathBuf,
    spec_valid: bool,
    spec: ProfileSpec<'a>,
}

impl Profile<'_> {
    pub fn check_spec(&mut self) -> Result<Vec<ModFileError>> {
        let errors = self.spec.check_modfiles()?;
        self.spec_valid = errors.len() == 0;
        
        Ok(errors)
    }

    pub fn is_downloaded(&self) -> bool {
        for requred_modfile in &self.spec.mod_files {
            if !(
                Path::new(&self.path)
                .join(requred_modfile.filename_with_modfile_id())
                .exists()
            ) {
                return false;
            }
        };
        
        true
    }

    // FIXME:
    // pub fn download_missing_modfiles(&self) {
    //     for requred_modfile in &self.spec.mod_files {
    //         if !(
    //             Path::new(&self.path)
    //             .join(requred_modfile.filename_with_modfile_id())
    //             .exists()
    //         ) {
    //             requred_modfile.download_to_dir(self.path);
    //         }
    //     };
    // }
}
