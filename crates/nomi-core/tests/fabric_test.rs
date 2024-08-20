use nomi_core::{
    game_paths::GamePaths,
    instance::{
        launch::{arguments::UserData, LaunchSettings},
        logs::PrintLogs,
        Profile,
    },
    loaders::fabric::Fabric,
    repository::java_runner::JavaRunner,
};

#[tokio::test]
async fn vanilla_test() {
    let subscriber = tracing_subscriber::fmt().pretty().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let game_paths = GamePaths {
        game: "./minecraft".into(),
        assets: "./minecraft/assets".into(),
        profile: "./minecraft/versions/1.20".into(),
        libraries: "./minecraft/libraries".into(),
    };

    let builder = Profile::builder()
        .version("1.20".into())
        .game_paths(game_paths.clone())
        .downloader(Box::new(Fabric::new("1.20", None::<String>, game_paths.clone()).await.unwrap()))
        // .instance(Inner::vanilla("1.20").await.unwrap())
        .name("1.20-fabric-test".into())
        .build();

    let _assets = builder.assets().await.unwrap();

    // _assets.download().await.unwrap();
    // builder.download().await.unwrap();

    let settings = LaunchSettings {
        java_runner: None,
        version: "1.20".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    let l = builder.launch_instance(settings, None);
    l.launch(game_paths, UserData::default(), &JavaRunner::default(), &PrintLogs)
        .await
        .unwrap();
}
