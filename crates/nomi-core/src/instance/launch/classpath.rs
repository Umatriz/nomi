use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::{rules::is_all_rules_satisfied, CLASSPATH_SEPARATOR};
use crate::repository::{manifest::ManifestLibrary, simple_lib::SimpleLib};

pub fn should_use_library(lib: &ManifestLibrary) -> Result<bool> {
    match lib.rules.as_ref() {
        Some(rules) => Ok(is_all_rules_satisfied(rules)?),
        None => Ok(true),
    }
}

pub fn classpath(
    jar_file: Option<PathBuf>,
    libraries_path: PathBuf,
    libraries: Vec<ManifestLibrary>,
    extra_libraries: Option<&Vec<SimpleLib>>,
) -> Result<String> {
    let mut classpath = vec![];

    if let Some(path) = jar_file {
        classpath.push(path)
    }

    for lib in libraries.iter() {
        if should_use_library(lib)? {
            if let Some(artifact) = lib.downloads.artifact.as_ref() {
                let lib_path = artifact.path.clone().context("LibPath must be Some()")?;

                let replaced_lib_path = if cfg!(target_os = "windows") {
                    lib_path.replace('/', "\\")
                } else {
                    lib_path
                };

                let final_lib_path = Path::new(&libraries_path).join(replaced_lib_path);

                classpath.push(final_lib_path);
            }

            if let Some(natives) = lib.downloads.classifiers.as_ref() {
                let native_option = match std::env::consts::OS {
                    "linux" => natives.natives_linux.as_ref(),
                    "windows" => natives.natives_windows.as_ref(),
                    "macos" => natives.natives_macos.as_ref(),
                    _ => unreachable!(),
                };

                if let Some(native_lib) = native_option {
                    let lib_path = native_lib.path.clone().context("LibPath must be Some()")?;

                    let replaced_lib_path = if cfg!(target_os = "windows") {
                        lib_path.replace('/', "\\")
                    } else {
                        lib_path
                    };

                    let final_lib_path = libraries_path.join(replaced_lib_path);

                    classpath.push(final_lib_path);
                }
            }
        }
    }

    if let Some(extra_libs) = extra_libraries {
        extra_libs.iter().for_each(|lib| {
            classpath.push(libraries_path.join(&lib.jar));
        })
    }
    let classpath_iter = classpath.iter().map(|p| p.display().to_string());

    let final_classpath =
        itertools::intersperse(classpath_iter, CLASSPATH_SEPARATOR.to_string()).collect::<String>();

    Ok(final_classpath)
}

#[cfg(test)]
mod tests {
    use crate::repository::manifest::Manifest;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        let fake_manifest: Manifest = reqwest::get("https://piston-meta.mojang.com/v1/packages/334b33fcba3c9be4b7514624c965256535bd7eba/1.18.2.json").await.unwrap().json().await.unwrap();

        let classpath = classpath(
            Some(PathBuf::from("test.jar")),
            PathBuf::from("test_libs"),
            fake_manifest.libraries,
            None,
        )
        .unwrap();

        println!("{}", classpath);
    }

    #[tokio::test]
    async fn extra_libraries() {
        let fake_manifest: Manifest = reqwest::get("https://piston-meta.mojang.com/v1/packages/334b33fcba3c9be4b7514624c965256535bd7eba/1.18.2.json").await.unwrap().json().await.unwrap();

        let artifact = "net.fabricmc:fabric-loader:0.14.22";

        let maven = crate::loaders::maven::MavenData::new(artifact);
        let simple_lib = SimpleLib::from(maven);

        let classpath = classpath(
            Some(PathBuf::from("test.jar")),
            PathBuf::from("test_libs"),
            fake_manifest.libraries,
            Some(&vec![simple_lib]),
        )
        .unwrap();

        println!("{}", classpath);
    }
}
