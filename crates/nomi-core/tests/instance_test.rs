use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    downloads::traits::Downloader,
    fs::write_toml_config,
    game_paths::GamePaths,
    instance::{
        launch::{arguments::UserData, LaunchSettings},
        logs::PrintLogs,
        Instance, Profile, ProfilePayload,
    },
    loaders::vanilla::Vanilla,
    repository::{java_runner::JavaRunner, manifest::VersionType},
};

#[tokio::test]
async fn instance_test() {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt().pretty().finish()).unwrap();

    let mut instance = Instance::new("cool-instance", 0);

    let paths = GamePaths::from_instance_path(instance.path(), "1.19.2");
    let profile = Profile::builder()
        .game_paths(paths.clone())
        .downloader(Box::new(Vanilla::new("1.19.2", paths.clone()).await.unwrap()))
        .name("Cool name".into())
        .version("1.19.2".into())
        .build();

    let launch_instance = profile.launch_instance(
        LaunchSettings {
            java_runner: None,
            version: "1.19.2".to_owned(),
            version_type: VersionType::Release,
        },
        None,
    );

    let (tx, _) = tokio::sync::mpsc::channel(1);

    let assets = profile.assets().await.unwrap();
    let io = assets.io();
    Box::new(assets).download(&tx).await;
    io.await.unwrap();

    let downloader = profile.downloader();
    let io = downloader.io();
    downloader.download(&tx).await;
    io.await.unwrap();

    let version_profile = VersionProfile {
        id: instance.next_id(),
        name: "Based".to_owned(),
        state: ProfileState::downloaded(launch_instance),
    };

    instance.add_profile(ProfilePayload::from_version_profile(&version_profile, &paths.profile_config()));

    write_toml_config(&version_profile, paths.profile_config()).await.unwrap();

    instance.write().await.unwrap();

    version_profile
        .launch(paths, UserData::default(), &JavaRunner::from_environment(), &PrintLogs)
        .await
        .unwrap();
}
