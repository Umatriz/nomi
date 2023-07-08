use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

use crate::{manifest::ManifestLibrary};
use super::rules::is_all_rules_satisfied;

pub fn should_use_library(lib: &ManifestLibrary) -> Result<bool> {
    let rules_opt = &lib.rules;
    return match rules_opt {
        None => Ok(true),
        Some(rules) => Ok(is_all_rules_satisfied(rules)?),
    };
}

pub fn create_classpath(
    jar_file: PathBuf,
    libraries_path: PathBuf,
    libraries: Vec<ManifestLibrary>,
) -> Result<String> {
    let mut classpath = jar_file.to_str().context("failed to convert classpath to string")?.to_string();

    for lib in libraries.iter() {
        let should_use = should_use_library(lib)?;
        if should_use {
            let artifact = &lib.downloads.artifact;
            let lib_path = artifact.as_ref().context("TODO: CHANGEME")?.path.clone().context("failed to clone lib path")?;
            let fixed_lib_path = Path::new(&libraries_path).join(lib_path.replace('/', "\\"));
            classpath = format!("{};{}", classpath, fixed_lib_path.to_str().context("failed to convert fixed_lib_path to string")?);
        }
    }

    return Ok(classpath);
}