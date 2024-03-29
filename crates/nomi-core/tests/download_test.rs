use nomi_core::instance::{Inner, InstanceBuilder};
use tracing::Level;

#[tokio::test]
async fn download_test() {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let instance = InstanceBuilder::new()
        .version("1.18.2".into())
        .libraries("./minecraft/libraries".into())
        .version_path("./minecraft/versions/1.18.2".into())
        .instance(Inner::vanilla("1.18.2").await.unwrap())
        .assets("./minecraft/assets".into())
        .game("./minecraft".into())
        .name("1.18.2-test".into())
        .build();

    instance.assets().await.unwrap().download().await.unwrap();

    instance.download().await.unwrap();
}
