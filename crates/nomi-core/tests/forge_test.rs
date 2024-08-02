use std::path::PathBuf;

use nomi_core::{
    configs::profile::{ProfileState, VersionProfile},
    downloads::traits::Downloader,
    game_paths::GamePaths,
    instance::{
        launch::{arguments::UserData, LaunchSettings},
        logs::PrintLogs,
        Instance,
    },
    loaders::{
        forge::{Forge, ForgeVersion},
        vanilla::Vanilla,
    },
    repository::java_runner::JavaRunner,
    MINECRAFT_DIR,
};

#[tokio::test]
async fn full_fabric_test() {
    let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

    let current = std::env::current_dir().unwrap();

    let (tx, _) = tokio::sync::mpsc::channel(100);

    let game_paths = GamePaths {
        version: PathBuf::from(MINECRAFT_DIR).join("versions").join("forge-test"),
        ..Default::default()
    };

    let instance = Instance::builder()
        .name("forge-test".into())
        .version("1.20.1".into())
        .game_paths(game_paths.clone())
        .instance(Box::new(Forge::new("1.20.1", ForgeVersion::Recommended, &game_paths).await.unwrap()))
        .build();

    let mc_dir = current.join("minecraft");

    // let vanilla = Box::new(Vanilla::new("1.20.1", game_paths.clone()).await.unwrap());
    // let io = vanilla.io();

    // vanilla.download(&tx).await;

    // io.await.unwrap();

    let settings = LaunchSettings {
        assets: mc_dir.join("assets"),
        game_dir: mc_dir.clone(),
        java_bin: JavaRunner::default(),
        libraries_dir: mc_dir.clone().join("libraries"),
        manifest_file: mc_dir.clone().join("versions/forge-test/1.20.1.json"),
        natives_dir: mc_dir.clone().join("versions/forge-test/natives"),
        version_jar_file: mc_dir.join("versions/forge-test/1.20.1.jar"),
        version: "1.20.1".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    let launch = instance.launch_instance(settings, None);

    Box::new(instance.assets().await.unwrap()).download(&tx).await;

    let instance = instance.instance();
    let ui_fut = instance.io();

    instance.download(&tx).await;

    ui_fut.await.unwrap();

    let profile = VersionProfile::builder()
        .id(1)
        .name("forge-test".into())
        .state(ProfileState::downloaded(launch))
        .build();

    dbg!(profile)
        .launch(UserData::default(), &JavaRunner::default(), &PrintLogs)
        .await
        .unwrap();
}
