use std::path::{Path, PathBuf};

use nomi_core::{
    configs::{
        profile::{VersionProfileBuilder, VersionProfilesConfig},
        read_toml_config, write_toml_config,
    },
    instance::{
        launch::{LaunchInstanceBuilder, LaunchSettings},
        Inner, InstanceBuilder,
    },
    repository::{java_runner::JavaRunner, username::Username},
};

use crate::args::{Cli, Loader};

pub async fn process_args(args: &Cli) -> anyhow::Result<()> {
    use crate::args::Command::*;
    match args {
        Cli {
            game_dir,
            command:
                Download {
                    name,
                    version,
                    loader,
                },
        } => download(name, game_dir, version, loader.as_ref()).await,
        Cli {
            game_dir,
            command: Launch { profile_id },
        } => todo!(),
        Cli {
            game_dir,
            command: List,
        } => todo!(),
    }
}

pub async fn download(
    name: &String,
    dir: &Path,
    game_version: &String,
    loader: Option<&Loader>,
) -> anyhow::Result<()> {
    let mc_dir = &dir.join("minecraft");
    let builder = InstanceBuilder::new()
        .name(name.to_string())
        .version(game_version.to_string())
        .assets(mc_dir.join("assets"))
        .game(mc_dir.clone())
        .libraries(mc_dir.join("libraries"))
        .version_path(mc_dir.join("versions").join(game_version));

    let instance = if let Some(loader) = loader {
        match loader {
            Loader::Fabric { version } => builder
                .instance(Inner::fabric(game_version, version.as_ref()).await?)
                .build(),
        }
    } else {
        builder
            .instance(Inner::vanilla(game_version).await?)
            .build()
    };

    instance.download().await?;

    if !instance.assets.exists() {
        instance.assets().await?.download().await?;
    }

    let confgis = dir.join(".nomi/configs");

    let mut profiles: VersionProfilesConfig = if confgis.join("Profiles.toml").exists() {
        read_toml_config(confgis.join("Profiles.toml")).await?
    } else {
        VersionProfilesConfig { profiles: vec![] }
    };

    let settings = LaunchSettings {
        access_token: None,
        username: Username::default(),
        uuid: None,
        assets: instance.assets.clone(),
        game_dir: instance.game.clone(),
        java_bin: JavaRunner::default(),
        libraries_dir: instance.libraries.clone(),
        manifest_file: instance.version_path.join(format!("{}.json", game_version)),
        natives_dir: instance.version_path.join("natives"),
        version_jar_file: instance.version_path.join(format!("{}.jar", game_version)),
        version: game_version.to_string(),
        version_type: "release".into(),
    };

    let launch_instance = instance.launch_instance(settings);
    let profile = VersionProfileBuilder::new()
        .id(profiles.create_id())
        .instance(launch_instance)
        .is_downloaded(true)
        .name(name.to_string())
        .build();
    profiles.add_profile(profile);

    write_toml_config(&profiles, confgis.join("Profiles.toml")).await?;

    Ok(())
}
