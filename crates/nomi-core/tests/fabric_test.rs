use nomi_core::instance::{launch::LaunchSettingsBuilder, InstanceBuilder};

#[tokio::test]
async fn vanilla_test() {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let builder = InstanceBuilder::new()
        .game("./minecraft")
        .libraries("./minecraft/libraries")
        .version("1.20")
        .version_path("./minecraft/instances/1.20")
        .fabric("1.20", None::<String>)
        .await
        .unwrap()
        .build();

    let assets = builder
        .assets("1.20")
        .await
        .unwrap()
        .indexes("./minecraft/assets/indexes")
        .objects("./minecraft/assets/objects")
        .build()
        .await
        .unwrap();

    assets.download().await.unwrap();
    builder.download().await.unwrap();

    let mc_dir = std::env::current_dir().unwrap().join("minecraft");
    let settings = LaunchSettingsBuilder::new()
        .access_token(None)
        .assets(mc_dir.join("assets"))
        .game_dir(mc_dir.clone())
        .java_bin("./java/jdk-17.0.8/bin/java.exe".into())
        .libraries_dir(mc_dir.clone().join("libraries"))
        .manifest_file(
            mc_dir
                .clone()
                .join("instances")
                .join("1.20")
                .join("1.20.json"),
        )
        .natives_dir(
            mc_dir
                .clone()
                .join("instances")
                .join("1.20")
                .join("natives"),
        )
        .username("ItWorks".to_string())
        .uuid(None)
        .version("1.20".to_string())
        .version_jar_file(mc_dir.join("instances").join("1.20").join("1.20.jar"))
        .version_type("release".to_string())
        .build();

    let l = builder.launch_instance(settings);
    l.launch().await.unwrap();
}
