use std::path::PathBuf;

use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    downloads::traits::Downloader,
    game_paths::GamePaths,
    instance::{
        launch::{arguments::UserData, LaunchSettings},
        logs::PrintLogs,
        InstanceProfileId, Profile,
    },
    loaders::forge::{Forge, ForgeVersion},
    repository::java_runner::JavaRunner,
    MINECRAFT_DIR,
};

#[tokio::test]
async fn forge_test() {
    let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

    let (tx, _) = tokio::sync::mpsc::channel(100);

    let game_paths = GamePaths {
        profile: PathBuf::from(MINECRAFT_DIR).join("versions").join("forge-test"),
        ..Default::default()
    };

    let instance = Profile::builder()
        .name("forge-test".into())
        .version("1.7.10".into())
        .game_paths(game_paths.clone())
        .downloader(Box::new(
            Forge::new("1.7.10", ForgeVersion::Recommended, game_paths.clone()).await.unwrap(),
        ))
        // .instance(Box::new(Vanilla::new("1.7.10", game_paths.clone()).await.unwrap()))
        .build();

    // let vanilla = Box::new(Vanilla::new("1.7.10", game_paths.clone()).await.unwrap());
    // let io = vanilla.io();

    // vanilla.download(&tx).await;

    // io.await.unwrap();

    let settings = LaunchSettings {
        java_runner: None,
        version: "1.7.10".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    let launch = instance.launch_instance(settings, None);

    let assets = instance.assets().await.unwrap();
    let assets_io = assets.io();
    Box::new(assets).download(&tx).await;
    assets_io.await.unwrap();

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
            &JavaRunner::path(PathBuf::from(
                "E:/programming/code/nomi/crates/nomi-core/.nomi/java/jdk8u422-b05/bin/javaw.exe",
            )),
            &PrintLogs,
        )
        .await
        .unwrap();
}
