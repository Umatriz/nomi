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

    let _settings = LaunchSettings {
        java_runner: None,
        version: "1.20".to_string(),
        version_type: nomi_core::repository::manifest::VersionType::Release,
    };

    // let l = builder.launch_instance(settings);
    // l.launch().await.unwrap();
}
