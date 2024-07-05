use nomi_core::{
    game_paths::GamePaths,
    instance::{
        launch::{arguments::UserData, LaunchSettings},
        InstanceBuilder,
    },
    loaders::fabric::Fabric,
    repository::java_runner::JavaRunner,
};

#[tokio::test]
async fn vanilla_test() {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let game_paths = GamePaths {
        game: "./minecraft".into(),
        assets: "./minecraft/assets".into(),
        version: "./minecraft/versions/1.20".into(),
        libraries: "./minecraft/libraries".into(),
    };

    let builder = InstanceBuilder::new()
        .version("1.20".into())
        .game_paths(game_paths.clone())
        .instance(Box::new(
            Fabric::new("1.20", None::<String>, game_paths)
                .await
                .unwrap(),
        ))
        // .instance(Inner::vanilla("1.20").await.unwrap())
        .name("1.20-fabric-test".into())
        .build();

    let _assets = builder.assets().await.unwrap();

    // _assets.download().await.unwrap();
    // builder.download().await.unwrap();

    let mc_dir = std::env::current_dir().unwrap().join("minecraft");

    let settings = LaunchSettings {
        assets: mc_dir.join("assets"),
        game_dir: mc_dir.clone(),
        java_bin: JavaRunner::default(),
        libraries_dir: mc_dir.clone().join("libraries"),
        manifest_file: mc_dir.clone().join("versions/1.20/1.20.json"),
        natives_dir: mc_dir.clone().join("versions/1.20/natives"),
        version_jar_file: mc_dir.join("versions/1.20/1.20.jar"),
        version: "1.20".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    let l = builder.launch_instance(settings, None);
    l.launch(UserData::default(), &JavaRunner::default())
        .await
        .unwrap();
}
