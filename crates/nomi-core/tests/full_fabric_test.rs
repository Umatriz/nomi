use nomi_core::{
    configs::profile::{ProfileState, VersionProfileBuilder, VersionProfilesConfig},
    downloads::traits::Downloader,
    game_paths::GamePaths,
    instance::{launch::LaunchSettings, InstanceBuilder},
    loaders::fabric::Fabric,
    repository::{java_runner::JavaRunner, username::Username},
};

#[tokio::test]
async fn full_fabric_test() {
    let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

    let current = std::env::current_dir().unwrap();

    let (tx, _) = tokio::sync::mpsc::channel(100);

    let game_paths = GamePaths {
        game: "./minecraft".into(),
        assets: "./minecraft/assets".into(),
        version: "./minecraft/versions/Full-fabric-test".into(),
        libraries: "./minecraft/libraries".into(),
    };

    let instance = InstanceBuilder::new()
        .name("Full-fabric-test".into())
        .version("1.19.4".into())
        .game_paths(game_paths.clone())
        .instance(Box::new(
            Fabric::new("1.19.4", None::<String>, game_paths)
                .await
                .unwrap(),
        ))
        .sender(tx.clone())
        .build();

    let mc_dir = current.join("minecraft");

    let settings = LaunchSettings {
        access_token: None,
        username: Username::new("ItWorks").unwrap(),
        uuid: None,
        assets: mc_dir.join("assets"),
        game_dir: mc_dir.clone(),
        java_bin: JavaRunner::default(),
        libraries_dir: mc_dir.clone().join("libraries"),
        manifest_file: mc_dir.clone().join("versions/Full-fabric-test/1.19.4.json"),
        natives_dir: mc_dir.clone().join("versions/Full-fabric-test/natives"),
        version_jar_file: mc_dir.join("versions/Full-fabric-test/1.19.4.jar"),
        version: "1.19.4".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    let launch = instance.launch_instance(settings, None);

    Box::new(instance.assets().await.unwrap())
        .download(tx)
        .await;
    instance.download().await.unwrap();

    let mock = VersionProfilesConfig { profiles: vec![] };
    let profile = VersionProfileBuilder::new()
        .id(mock.create_id())
        .name("Full-fabric-test".into())
        .state(ProfileState::downloaded(launch))
        .build();

    dbg!(profile).launch().await.unwrap();
}
