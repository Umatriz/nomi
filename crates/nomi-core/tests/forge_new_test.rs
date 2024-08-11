use std::path::PathBuf;

use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    game_paths::GamePaths,
    instance::{
        launch::{arguments::UserData, LaunchSettings},
        logs::PrintLogs,
        InstanceProfileId, Profile,
    },
    loaders::forge::{Forge, ForgeVersion},
    repository::java_runner::JavaRunner,
    DOT_NOMI_JAVA_EXECUTABLE,
};

#[tokio::test]
async fn forge_test() {
    let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt().pretty().finish());

    let (tx, _) = tokio::sync::mpsc::channel(100);

    let game_paths = GamePaths::from_id(InstanceProfileId::ZERO);

    let instance = Profile::builder()
        .name("forge-test".into())
        .version("1.20.1".into())
        .game_paths(game_paths.clone())
        .downloader(Box::new(
            Forge::new("1.20.1", ForgeVersion::Recommended, game_paths.clone()).await.unwrap(),
        ))
        // .downloader(Box::new(Vanilla::new("1.20.1", game_paths.clone()).await.unwrap()))
        .build();

    // let vanilla = Box::new(Vanilla::new("1.20.1", game_paths.clone()).await.unwrap());
    // let io = vanilla.io();

    // vanilla.download(&tx).await;

    // io.await.unwrap();

    let settings = LaunchSettings {
        java_runner: None,

        version: "1.20.1".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    let launch = instance.launch_instance(settings, None);

    // let assets = instance.assets().await.unwrap();
    // let assets_io = assets.io();
    // Box::new(assets).download(&tx).await;
    // assets_io.await.unwrap();

    let instance = instance.downloader();
    let io_fut = instance.io();

    instance.download(&tx).await;

    io_fut.await.unwrap();

    let profile = VersionProfile::builder()
        .id(InstanceProfileId::ZERO)
        .name("forge-test".into())
        .state(ProfileState::downloaded(launch))
        .build();

    dbg!(profile)
        .launch(
            game_paths,
            UserData::default(),
            &JavaRunner::path(PathBuf::from(DOT_NOMI_JAVA_EXECUTABLE)),
            &PrintLogs,
        )
        .await
        .unwrap();
}
