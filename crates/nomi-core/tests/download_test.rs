use nomi_core::instance::InstanceBuilder;
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
        .libraries("./minecraft/libraries")
        .version_path("./minecraft/versions/1.18.2")
        .vanilla("1.18.2")
        .await
        .unwrap()
        .build();

    instance
        .assets("1.18.2")
        .await
        .unwrap()
        .indexes("./minecraft/assets/indexes")
        .objects("./minecraft/assets/objects")
        .build()
        .await
        .unwrap();

    instance.download().await.unwrap();
}
