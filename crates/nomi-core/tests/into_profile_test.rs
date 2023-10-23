use nomi_core::{
    configs::profile::VersionProfilesConfig,
    instance::{Inner, InstanceBuilder},
    repository::{java_runner::JavaRunner, username::Username},
};

#[tokio::test]
async fn into_profile_test() {
    let current = std::env::current_dir().unwrap();
    let builder = InstanceBuilder::new()
        .version("1.20".into())
        .libraries(current.join("./minecraft/libraries"))
        .version_path(current.join("./minecraft/versions/1.20"))
        .instance(Inner::fabric("1.20", None::<String>).await.unwrap())
        .assets(current.join("./minecraft/assets"))
        .game(current.join("./minecraft"))
        .name("1.20-fabric-test".into())
        .build();

    // let profile = builder.into_profile(
    //     &VersionProfilesConfig { profiles: vec![] },
    //     "release".into(),
    //     false,
    // );
    // profile
    //     .into_launch(Username::new("test").unwrap(), JavaRunner::STR, None, None)
    //     .launch()
    //     .await
    //     .unwrap();
}
