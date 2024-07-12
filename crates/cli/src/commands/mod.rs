use std::path::{Path, PathBuf};

use nomi_core::{
    configs::{
        profile::{VersionProfileBuilder, VersionProfilesConfig},
        read_toml_config,
        user::Settings,
        write_toml_config,
    },
    instance::{launch::LaunchSettings, Inner, InstanceBuilder},
    repository::{java_runner::JavaRunner, username::Username},
};
use tracing::warn;

use crate::{
    args::{Cli, Loader},
    error::Error,
};

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
        } => {
            let _ = launch(game_dir, profile_id).await?;
            Ok(())
        }
        Cli {
            game_dir,
            command:
                Register {
                    username,
                    access_token,
                    java_bin,
                    uuid,
                },
        } => {
            register(
                game_dir,
                username,
                access_token.as_ref(),
                java_bin.as_ref(),
                uuid.as_ref(),
            )
            .await
        }
        Cli {
            game_dir,
            command: List,
        } => list(game_dir).await,
    }
}

pub async fn download(
    name: &String,
    dir: &Path,
    game_version: &String,
    loader: Option<&Loader>,
) -> anyhow::Result<()> {
    let dir = match dir.is_absolute() {
        true => dir.to_path_buf(),
        false => {
            warn!("`GAME_DIR` is not absolute. Adding to the current dir");
            std::env::current_dir()?.join(dir)
        }
    };
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

    // if !instance.assets.exists() {
    instance.assets().await?.download().await?;
    // }

    let configs = dir.join(".nomi/configs");

    let mut profiles: VersionProfilesConfig = if configs.join("Profiles.toml").exists() {
        read_toml_config(configs.join("Profiles.toml")).await?
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

    let launch_instance = instance.launch_instance(
        settings,
        Some(vec!["-Xms2G".to_string(), "-Xmx4G".to_string()]),
    );
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

pub async fn launch(dir: &Path, profile_id: &i32) -> anyhow::Result<i32> {
    if !dir.join(".nomi/configs/User.toml").exists()
        || !dir.join(".nomi/configs/Profiles.toml").exists()
    {
        return Err(Error::General(
            "`User.toml` and `Profiles.toml` not found\nRun `register` and `download` before"
                .into(),
        )
        .into());
    }

    let profiles_cfg: VersionProfilesConfig =
        read_toml_config(dir.join(".nomi/configs/Profiles.toml")).await?;

    let user_cfg: Settings = read_toml_config(dir.join(".nomi/configs/User.toml")).await?;

    let p = profiles_cfg
        .profiles
        .into_iter()
        .find(|p| &p.id == profile_id);

    let Some(mut profile) = p else {
        return Err(
            Error::General("No such profile\nRun `list` to see installed versions".into()).into(),
        );
    };

    /*
       TODO: `jvm_args` don't works
    */

    profile.instance.set_username(user_cfg.username);
    profile.instance.set_access_token(user_cfg.access_token);
    profile.instance.set_uuid(user_cfg.uuid);

    profile.launch().await
}

pub async fn register(
    dir: &Path,
    username: &str,
    access_token: Option<&String>,
    java_bin: Option<&PathBuf>,
    uuid: Option<&String>,
) -> anyhow::Result<()> {
    let dir = match dir.is_absolute() {
        true => dir.to_path_buf(),
        false => {
            warn!("`GAME_DIR` is not absolute. Adding to the current dir");
            std::env::current_dir()?.join(dir)
        }
    };

    let settings = Settings {
        username: Username::new(username)?,
        access_token: access_token.map(String::from),
        java_bin: java_bin.map(|p| JavaRunner::path(p.clone())),
        uuid: uuid.map(String::from),
    };

    write_toml_config(&settings, dir.join(".nomi/configs/User.toml")).await?;

    Ok(())
}

pub async fn list(dir: &Path) -> anyhow::Result<()> {
    let dir = match dir.is_absolute() {
        true => dir.to_path_buf(),
        false => {
            warn!("`GAME_DIR` is not absolute. Adding to the current dir");
            std::env::current_dir()?.join(dir)
        }
    };

    if !dir.join(".nomi/configs/User.toml").exists()
        || !dir.join(".nomi/configs/Profiles.toml").exists()
    {
        return Err(Error::General(
            "`User.toml` and `Profiles.toml` not found\nRun `register` and `download` before"
                .into(),
        )
        .into());
    }

    let prof: VersionProfilesConfig =
        read_toml_config(dir.join(".nomi/configs/Profiles.toml")).await?;

    prof.profiles.iter().for_each(|p| {
        println!("{}: {}", p.name, p.id);
    });

    Ok(())
}
