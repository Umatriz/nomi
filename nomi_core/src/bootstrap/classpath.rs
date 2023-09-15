use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::rules::is_all_rules_satisfied;
use crate::repository::manifest::ManifestLibrary;

pub fn should_use_library(lib: &ManifestLibrary) -> Result<bool> {
    let rules_opt = &lib.rules;

    match rules_opt {
        Some(rules) => Ok(is_all_rules_satisfied(rules)?),
        None => Ok(true),
    }
}

pub fn create_classpath(
    jar_file: PathBuf,
    libraries_path: PathBuf,
    libraries: Vec<ManifestLibrary>,
) -> Result<String> {
    let mut classpath = jar_file
        .to_str()
        .context("failed to convert classpath to string")?
        .to_string();

    for lib in libraries.iter() {
        let should_use = should_use_library(lib)?;
        if should_use {
            let artifact = &lib.downloads.artifact;
            let lib_path = artifact.as_ref().unwrap().path.clone();
            let fixed_lib_path =
                Path::new(&libraries_path).join(lib_path.unwrap().replace('/', "\\"));
            classpath = format!("{};{}", classpath, fixed_lib_path.to_str().unwrap());
        }
    }

    Ok(classpath)
}
