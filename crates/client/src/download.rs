use std::sync::mpsc::Sender;

use nomi_core::{
    configs::{
        profile::{VersionProfile, VersionProfileBuilder, VersionProfilesConfig},
        read_toml_config, write_toml_config,
    },
    instance::{launch::LaunchSettings, Inner, InstanceBuilder},
    repository::{java_runner::JavaRunner, username::Username},
};

use crate::{utils::Crash, Loader};

pub fn spawn_download(
    tx: Sender<VersionProfile>,
    name: String,
    version: String,
    loader: Loader,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let data = try_download(name, version, loader).await.crash();
        let _ = tx.send(data);
    })
}

async fn try_download(
    name: String,
    version: String,
    loader: Loader,
) -> anyhow::Result<VersionProfile> {
    // return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Error").into());
    let current = std::env::current_dir()?;
    let mc_dir: std::path::PathBuf = current.join("minecraft");
    let builder = InstanceBuilder::new()
        .name(name.to_string())
        .version(version.clone())
        .assets(mc_dir.join("assets"))
        .game(mc_dir.clone())
        .libraries(mc_dir.join("libraries"))
        .version_path(mc_dir.join("versions").join(&version));

    let instance = match loader {
        Loader::Vanilla => builder.instance(Inner::vanilla(&version).await?).build(),
        Loader::Fabric => builder
            .instance(Inner::fabric(&version, None::<String>).await?)
            .build(),
    };

    instance.download().await?;
    instance.assets().await?.download().await?;

    let confgis = current.join(".nomi/configs");

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
        manifest_file: instance.version_path.join(format!("{}.json", &version)),
        natives_dir: instance.version_path.join("natives"),
        version_jar_file: instance.version_path.join(format!("{}.jar", &version)),
        version,
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
    profiles.add_profile(profile.clone());

    write_toml_config(&profiles, confgis.join("Profiles.toml")).await?;

    Ok(profile)
}