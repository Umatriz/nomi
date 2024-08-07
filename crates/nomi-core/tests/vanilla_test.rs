use nomi_core::{instance::launch::LaunchSettings, repository::java_runner::JavaRunner};

#[tokio::test]
async fn vanilla_test() {
    let subscriber = tracing_subscriber::fmt().pretty().with_max_level(tracing::Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // let builder = InstanceBuilder::new()
    //     .libraries("./minecraft/libraries")
    //     .version("1.20")
    //     .version_path("./minecraft/instances/1.20")
    //     .vanilla("1.20")
    //     .await
    //     .unwrap()
    //     .build();

    // let assets = builder
    //     .assets("1.20")
    //     .await
    //     .unwrap()
    //     .indexes("./minecraft/assets/indexes")
    //     .objects("./minecraft/assets/objects")
    //     .build()
    //     .await
    //     .unwrap();

    // // assets.download().await.unwrap();
    // // builder.download().await.unwrap();

    let mc_dir = std::env::current_dir().unwrap().join("minecraft");

    let _settings = LaunchSettings {
        assets: mc_dir.join("assets"),
        game_dir: mc_dir.clone(),
        java_bin: JavaRunner::default(),
        libraries_dir: mc_dir.clone().join("libraries"),
        manifest_file: mc_dir.clone().join("versions/1.20/1.19.4.json"),
        natives_dir: mc_dir.clone().join("versions/1.20/natives"),
        version_jar_file: mc_dir.join("versions/1.20/1.20.jar"),
        version: "1.20".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    // let l = builder.launch_instance(settings);
    // l.launch().await.unwrap();
}
