// use std::sync::mpsc::Sender;

// use nomi_core::{
//     configs::profile::{VersionProfile, VersionProfileBuilder, VersionProfilesConfig},
//     downloads::{
//         traits::{DownloadResult, Downloader, DownloaderIO, DownloaderIOExt},
//         DownloadQueue,
//     },
//     fs::{read_toml_config, write_toml_config},
//     game_paths::GamePaths,
//     instance::{launch::LaunchSettings, InstanceBuilder},
//     loaders::{fabric::Fabric, vanilla::Vanilla},
//     repository::{java_runner::JavaRunner, username::Username},
// };

// use crate::{utils::Crash, Loader};

// pub fn spawn_download(
//     tx: Sender<VersionProfile>,
//     name: String,
//     version: String,
//     loader: Loader,
//     progress_tx: tokio::sync::mpsc::Sender<DownloadResult>,
//     total_tx: tokio::sync::mpsc::Sender<u32>,
// ) -> tokio::task::JoinHandle<()> {
//     tokio::spawn(async move {
//         let data = try_download(name, version, loader, progress_tx.clone(), total_tx)
//             .await
//             .crash();

//         let _ = tx.send(data);
//     })
// }

// async fn try_download(
//     name: String,
//     version: String,
//     loader: Loader,
//     sender: tokio::sync::mpsc::Sender<DownloadResult>,
//     total_tx: tokio::sync::mpsc::Sender<u32>,
// ) -> anyhow::Result<VersionProfile> {
//     // let current = std::env::current_dir()?;
//     // let mc_dir: std::path::PathBuf = current.join("minecraft");

//     // let game_paths = GamePaths {
//     //     game: mc_dir.clone(),
//     //     assets: mc_dir.join("assets"),
//     //     version: mc_dir.join("versions").join(&version),
//     //     libraries: mc_dir.join("libraries"),
//     // };

//     // let builder = InstanceBuilder::new()
//     //     .name(name.to_string())
//     //     .version(version.clone())
//     //     .game_paths(game_paths.clone())
//     //     .sender(sender.clone());

//     // let instance = match loader {
//     //     Loader::Vanilla => builder
//     //         .instance(Box::new(Vanilla::new(&version, game_paths.clone()).await?))
//     //         .build(),
//     //     Loader::Fabric => builder
//     //         .instance(Box::new(
//     //             Fabric::new(&version, None::<String>, game_paths).await?,
//     //         ))
//     //         .build(),
//     // };

//     // let settings = LaunchSettings {
//     //     access_token: None,
//     //     username: Username::default(),
//     //     uuid: None,
//     //     assets: instance.game_paths.assets.clone(),
//     //     game_dir: instance.game_paths.game.clone(),
//     //     java_bin: JavaRunner::default(),
//     //     libraries_dir: instance.game_paths.libraries.clone(),
//     //     manifest_file: instance
//     //         .game_paths
//     //         .version
//     //         .join(format!("{}.json", &version)),
//     //     natives_dir: instance.game_paths.version.join("natives"),
//     //     version_jar_file: instance
//     //         .game_paths
//     //         .version
//     //         .join(format!("{}.jar", &version)),
//     //     version,
//     //     version_type: "release".into(),
//     // };

//     // let launch_instance = instance.launch_instance(
//     //     settings,
//     //     Some(vec!["-Xms2G".to_string(), "-Xmx4G".to_string()]),
//     // );

//     // let assets = instance.assets().await?;

//     // assets.get_io().io().await?;

//     // let downloader: Box<dyn Downloader<Data = DownloadResult>> =
//     //     instance.instance.into_downloader();

//     // let downloader = DownloadQueue::new()
//     //     .with_downloader(assets)
//     //     .with_downloader_dyn(downloader);

//     // let _ = total_tx.send(downloader.total()).await;

//     // Box::new(downloader).download(sender).await;

//     // // instance.download().await?;

//     // let confgis = current.join(".nomi/configs");

//     // let mut profiles: VersionProfilesConfig = if confgis.join("Profiles.toml").exists() {
//     //     read_toml_config(confgis.join("Profiles.toml")).await?
//     // } else {
//     //     VersionProfilesConfig { profiles: vec![] }
//     // };

//     // let profile = VersionProfileBuilder::new()
//     //     .id(profiles.create_id())
//     //     .instance(launch_instance)
//     //     .is_downloaded(true)
//     //     .name(name.to_string())
//     //     .build();
//     // profiles.add_profile(profile.clone());

//     // write_toml_config(&profiles, confgis.join("Profiles.toml")).await?;

//     // Ok(profile)
// }
