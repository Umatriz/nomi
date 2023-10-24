use nomi_core::{
    configs::profile::{VersionProfileBuilder, VersionProfilesConfig},
    instance::{launch::LaunchSettings, Inner, InstanceBuilder},
    repository::{java_runner::JavaRunner, username::Username},
};

#[tokio::test]
async fn full_fabric_test() {
    let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt().finish());

    let current = std::env::current_dir().unwrap();

    let instance = InstanceBuilder::new()
        .name("Full-fabric-test".into())
        .version("1.19.4".into())
        .version_path("./minecraft/versions/Full-fabric-test".into())
        .game("./minecraft".into())
        .libraries("./minecraft/libraries".into())
        .assets("./minecraft/assets".into())
        .instance(Inner::fabric("1.19.4", None::<String>).await.unwrap())
        .build();

    instance.assets().await.unwrap().download().await.unwrap();
    instance.download().await.unwrap();

    let mc_dir = current.join("minecraft");

    let settings = LaunchSettings {
        access_token: None,
        username: Username::new("ItWorks").unwrap(),
        uuid: None,
        assets: mc_dir.join("assets"),
        game_dir: mc_dir.clone(),
        java_bin: JavaRunner::default(),
        libraries_dir: mc_dir.clone().join("libraries"),
        manifest_file: mc_dir.clone().join("versions/Full-fabric-test/1.19.4.json"),
        natives_dir: mc_dir.clone().join("versions/Full-fabric-test/natives"),
        version_jar_file: mc_dir.join("versions/Full-fabric-test/1.19.4.jar"),
        version: "1.19.4".to_string(),
        version_type: "release".to_string(),
    };

    let launch = instance.launch_instance(settings);

    let mock = VersionProfilesConfig { profiles: vec![] };
    let profile = VersionProfileBuilder::new()
        .id(mock.create_id())
        .name("Full-fabric-test".into())
        .instance(launch)
        .is_downloaded(true)
        .build();

    dbg!(profile).launch().await.unwrap();
}
