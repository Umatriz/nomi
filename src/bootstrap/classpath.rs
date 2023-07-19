use std::path::{Path, PathBuf};

use super::rules::is_all_rules_satisfied;
use crate::{loaders::profile::LoaderLibrary, manifest::ManifestLibrary};

// TODO: Add loaders support

pub fn should_use_library(lib: &ManifestLibrary) -> bool {
    let rules_opt = &lib.rules;
    if !rules_opt.is_some() {
        return true;
    }

    let rules = rules_opt.as_ref().unwrap();
    is_all_rules_satisfied(rules)
}

pub fn create_classpath(
    jar_file: PathBuf,
    libraries_path: PathBuf,
    libraries: Vec<ManifestLibrary>,
    loader_libs: Vec<PathBuf>,
) -> String {
    let mut classpath = jar_file.to_str().unwrap().to_string();

    for lib in libraries.iter() {
        let should_use = should_use_library(lib);
        if should_use {
            let artifact = &lib.downloads.artifact;
            let lib_path = artifact.as_ref().unwrap().path.clone();
            let fixed_lib_path = Path::new(&libraries_path).join(lib_path.unwrap());
            classpath = format!("{};{}", classpath, fixed_lib_path.to_str().unwrap());
        }
    }

    for lib in loader_libs.iter() {
        classpath = format!("{};{}", classpath, lib.to_string_lossy());
    }

    classpath
}
