use nomi_core::{
    configs::profile::VersionProfilesConfig,
    instance::{launch::LaunchSettingsBuilder, Inner, InstanceBuilder},
    repository::{java_runner::JavaRunner, username::Username},
};

#[tokio::test]
async fn vanilla_test() {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let builder = InstanceBuilder::new()
        .version("1.20".into())
        .libraries("./minecraft/libraries".into())
        .version_path("./minecraft/versions/1.20".into())
        .instance(Inner::fabric("1.20", None::<String>).await.unwrap())
        .assets("./minecraft/assets".into())
        .game("./minecraft".into())
        .name("1.20-fabric-test".into())
        .build();

    let assets = builder.assets().await.unwrap();

    // assets.download().await.unwrap();
    // builder.download().await.unwrap();

    builder
        .into_profile(
            &VersionProfilesConfig { profiles: vec![] },
            "release".into(),
            true,
        )
        .into_launch(Username::new("test").unwrap(), JavaRunner::STR, None, None)
        .launch()
        .await
        .unwrap();

    // let mc_dir = std::env::current_dir().unwrap().join("minecraft");
    // let settings = LaunchSettingsBuilder::new()
    //     .access_token(None)
    //     .assets(mc_dir.join("assets"))
    //     .game_dir(mc_dir.clone())
    //     .java_bin(JavaRunner::Path("./java/jdk-17.0.8/bin/java.exe".into()))
    //     .libraries_dir(mc_dir.clone().join("libraries"))
    //     .manifest_file(
    //         mc_dir
    //             .clone()
    //             .join("instances")
    //             .join("1.20")
    //             .join("1.20.json"),
    //     )
    //     .natives_dir(
    //         mc_dir
    //             .clone()
    //             .join("instances")
    //             .join("1.20")
    //             .join("natives"),
    //     )
    //     .username(Username::new("ItWorks").unwrap())
    //     .uuid(None)
    //     .version("1.20".to_string())
    //     .version_jar_file(mc_dir.join("instances").join("1.20").join("1.20.jar"))
    //     .version_type("release".to_string())
    //     .build();

    // let l = builder.launch_instance(settings);
    // l.launch().await.unwrap();
}
