use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    downloads::traits::Downloader,
    game_paths::GamePaths,
    instance::{
        launch::{arguments::UserData, LaunchSettings},
        logs::PrintLogs,
        InstanceProfileId, Profile,
    },
    loaders::fabric::Fabric,
    repository::java_runner::JavaRunner,
};

#[tokio::test]
async fn full_fabric_test() {
    let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

    let (tx, _) = tokio::sync::mpsc::channel(100);

    let game_paths = GamePaths {
        game: "./minecraft".into(),
        assets: "./minecraft/assets".into(),
        profile: "./minecraft/versions/Full-fabric-test".into(),
        libraries: "./minecraft/libraries".into(),
    };

    let instance = Profile::builder()
        .name("Full-fabric-test".into())
        .version("1.19.4".into())
        .game_paths(game_paths.clone())
        .downloader(Box::new(Fabric::new("1.19.4", None::<String>, game_paths.clone()).await.unwrap()))
        .build();

    let settings = LaunchSettings {
        java_runner: None,
        version: "1.19.4".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    let launch = instance.launch_instance(settings, None);

    Box::new(instance.assets().await.unwrap()).download(&tx).await;

    let instance = instance.downloader();
    let ui_fut = instance.io();

    instance.download(&tx).await;

    ui_fut.await.unwrap();

    let profile = VersionProfile::builder()
        .id(InstanceProfileId::ZERO)
        .name("Full-fabric-test".into())
        .state(ProfileState::downloaded(launch))
        .build();

    dbg!(profile)
        .launch(game_paths, UserData::default(), &JavaRunner::default(), &PrintLogs)
        .await
        .unwrap();
}
