use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::{rules::is_all_rules_satisfied, CLASSPATH_SEPARATOR};
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
    let mut classpath = jar_file.to_string_lossy().to_string();

    for lib in libraries.iter() {
        if should_use_library(lib)? {
            let artifact = lib
                .downloads
                .artifact
                .as_ref()
                .context("artifact must be Some()")?;
            let lib_path = artifact.path.clone().context("LibPath must be Some()")?;

            let replaced_lib_path = if cfg!(target_os = "windows") {
                lib_path.replace('/', "\\")
            } else {
                lib_path
            };

            let final_lib_path = Path::new(&libraries_path).join(replaced_lib_path);

            classpath.push_str(
                format!(
                    "{}{}",
                    CLASSPATH_SEPARATOR,
                    final_lib_path.to_string_lossy()
                )
                .as_str(),
            );

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

                    let final_lib_path = Path::new(&libraries_path).join(replaced_lib_path);

                    classpath.push_str(
                        format!(
                            "{}{}",
                            CLASSPATH_SEPARATOR,
                            final_lib_path.to_string_lossy()
                        )
                        .as_str(),
                    );
                }
            }
        }
    }

    Ok(classpath)
}

#[cfg(test)]
mod tests {
    use crate::repository::manifest::Manifest;

    use super::*;

    #[tokio::test]
    async fn it_works() {
        let fake_manifest: Manifest = reqwest::get("https://piston-meta.mojang.com/v1/packages/334b33fcba3c9be4b7514624c965256535bd7eba/1.18.2.json").await.unwrap().json().await.unwrap();

        let classpath = create_classpath(
            PathBuf::from("test.jar"),
            PathBuf::from("test_libs"),
            fake_manifest.libraries,
        )
        .unwrap();

        println!("{}", classpath);
    }
}
