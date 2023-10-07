use nomi_core::loaders::{instance::InstanceBuilder, vanilla::Vanilla};
use tracing::Level;

#[tokio::test]
async fn download_test() {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let instance = InstanceBuilder::new()
        .version("1.18.2")
        .game("./minecraft")
        .libraries("./minecraft/libraries")
        .version_path("./minecraft/versions/1.18.2")
        .instance(async { Vanilla::new("1.18.2").await })
        .build()
        .await
        .unwrap();

    instance.download().await.unwrap();
}
