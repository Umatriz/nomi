use std::{
    error::Error,
    fmt::Display,
    future::Future,
    io::Write,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread::JoinHandle,
};

use eframe::egui;
use nomi_core::{
    configs::{
        profile::{VersionProfile, VersionProfileBuilder, VersionProfilesConfig},
        read_toml_config, read_toml_config_sync,
        user::Settings,
        write_toml_config,
    },
    instance::{launch::LaunchSettings, Inner, InstanceBuilder},
    repository::{
        java_runner::JavaRunner,
        launcher_manifest::{LauncherManifest, LauncherManifestVersion},
        username::Username,
    },
    utils::state::{launcher_manifest_state_try_init, ManifestState, LAUNCHER_MANIFEST_STATE},
};

fn main() {
    let subscriber = tracing_subscriber::fmt().compact().finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create Runtime");

    let _enter = runtime.enter();

    std::thread::spawn(move || {
        runtime.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        })
    });

    let _ = eframe::run_native(
        "Nomi",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(App::new().unwrap())),
    );
}

pub struct App {
    tx: Sender<VersionProfile>,
    rx: Receiver<VersionProfile>,
    tasks: Vec<(tokio::task::JoinHandle<()>, String)>,

    profiles: VersionProfilesConfig,
    settings: Settings,
    version_manifest: &'static ManifestState,

    release_versions: Vec<&'static LauncherManifestVersion>,

    username_buf: String,
    profile_name_buf: String,
    selected_version_buf: usize,
    loader_buf: Loader,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let profiles =
            read_toml_config_sync::<VersionProfilesConfig>("./.nomi/configs/Profiles.toml");
        let settings_res = read_toml_config_sync::<Settings>("./.nomi/configs/User.toml");
        let settings = settings_res.unwrap_or_default();

        let state = pollster::block_on(
            LAUNCHER_MANIFEST_STATE.get_or_try_init(launcher_manifest_state_try_init),
        )?;

        Ok(Self {
            release_versions: state
                .launcher
                .versions
                .iter()
                .filter(|i| i.version_type == *"release")
                .collect::<Vec<_>>(),
            version_manifest: state,
            tx,
            rx,
            tasks: Default::default(),
            profiles: profiles.unwrap_or_default(),
            username_buf: settings.username.get().to_string(),
            settings,
            profile_name_buf: Default::default(),
            selected_version_buf: Default::default(),
            loader_buf: Loader::Vanilla,
        })
    }

    pub fn spawn_download(
        tx: Sender<VersionProfile>,
        ctx: egui::Context,
        name: String,
        version: String,
        loader: Loader,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let data = Self::try_download(name, version, loader)
                .await
                .map_err(|err| {
                    let mut file = std::fs::File::create("./CRASH_REPORT.txt")
                        .expect("Cannot create CRASH_REPORT.txt");
                    file.write_all("App::download() paniced with following error:".as_bytes())
                        .unwrap();
                    file.write_all(format!("{:#?}", &err).as_bytes()).unwrap();
                    err
                })
                .unwrap();
            let _ = tx.send(data);
            ctx.request_repaint()
        })
    }

    async fn try_download(
        name: String,
        version: String,
        loader: Loader,
    ) -> anyhow::Result<VersionProfile> {
        let current = std::env::current_dir()?;
        let mc_dir: std::path::PathBuf = current.join("minecraft");
        let builder = InstanceBuilder::new()
            .name(name.to_string())
            .version(version.clone())
            .assets(mc_dir.join("assets"))
            .game(mc_dir.clone())
            .libraries(mc_dir.join("libraries"))
            .version_path(mc_dir.join("versions").join(&version));

        let instance = match loader {
            Loader::Vanilla => builder.instance(Inner::vanilla(&version).await?).build(),
            Loader::Fabric => builder
                .instance(Inner::fabric(&version, None::<String>).await?)
                .build(),
        };

        instance.download().await?;
        instance.assets().await?.download().await?;

        let confgis = current.join(".nomi/configs");

        let mut profiles: VersionProfilesConfig = if confgis.join("Profiles.toml").exists() {
            read_toml_config(confgis.join("Profiles.toml")).await?
        } else {
            VersionProfilesConfig { profiles: vec![] }
        };

        let settings = LaunchSettings {
            access_token: None,
            username: Username::default(),
            uuid: None,
            assets: instance.assets.clone(),
            game_dir: instance.game.clone(),
            java_bin: JavaRunner::default(),
            libraries_dir: instance.libraries.clone(),
            manifest_file: instance.version_path.join(format!("{}.json", &version)),
            natives_dir: instance.version_path.join("natives"),
            version_jar_file: instance.version_path.join(format!("{}.jar", &version)),
            version,
            version_type: "release".into(),
        };

        let launch_instance = instance.launch_instance(
            settings,
            Some(vec!["-Xms2G".to_string(), "-Xmx4G".to_string()]),
        );

        let profile = VersionProfileBuilder::new()
            .id(profiles.create_id())
            .instance(launch_instance)
            .is_downloaded(true)
            .name(name.to_string())
            .build();
        profiles.add_profile(profile.clone());

        write_toml_config(&profiles, confgis.join("Profiles.toml")).await?;

        Ok(profile)
    }
}

#[derive(PartialEq, Clone)]
pub enum Loader {
    Vanilla,
    Fabric,
}

impl Display for Loader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Loader::Vanilla => f.write_str("Vanilla"),
            Loader::Fabric => f.write_str("Fabric"),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // if let Some(err) = self.error.as_ref() {
            //     ui.label(err);
            // }
            if let Ok(data) = self.rx.try_recv() {
                self.profiles.add_profile(data);
            }

            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username_buf);

            ui.label("Create new profile");
            ui.label("Profile name:");
            ui.text_edit_singleline(&mut self.profile_name_buf);
            egui::ComboBox::from_label("Select version")
                .selected_text(&self.release_versions[self.selected_version_buf].id)
                .show_ui(ui, |ui| {
                    for i in 0..self.release_versions.len() {
                        let value = ui.selectable_value(
                            &mut &self.release_versions[i],
                            &self.release_versions[self.selected_version_buf],
                            &self.release_versions[i].id,
                        );
                        if value.clicked() {
                            self.selected_version_buf = i
                        }
                    }
                });

            ui.horizontal(|ui| {
                ui.radio_value(&mut self.loader_buf, Loader::Vanilla, "Vanilla");
                ui.radio_value(&mut self.loader_buf, Loader::Fabric, "Fabric")
            });
            ui.label("You must install vanilla before Fabric");

            if ui.button("Create and download").clicked() {
                let handle = Self::spawn_download(
                    self.tx.clone(),
                    ctx.clone(),
                    self.profile_name_buf.clone(),
                    self.release_versions[self.selected_version_buf].id.clone(),
                    self.loader_buf.clone(),
                );
                self.tasks.push((
                    handle,
                    format!(
                        "Downloading version {} - {}",
                        self.release_versions[self.selected_version_buf].id, self.loader_buf
                    ),
                ))
            }

            if !self.tasks.is_empty() {
                for (handle, name) in &self.tasks {
                    if !handle.is_finished() {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(format!("Task {} is running", name))
                        });
                    }
                }
            }

            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                ui.vertical(|ui| {
                    for profile in self.profiles.profiles.clone() {
                        ui.horizontal(|ui| {
                            ui.label(profile.name.to_string());
                            ui.label(format!("Id: {}", profile.id));
                            if ui.button("Launch").clicked() {
                                let (tx, rx) = std::sync::mpsc::channel();
                                let username = self.username_buf.clone();
                                let access_token = self.settings.access_token.clone();
                                let uuid = self.settings.uuid.clone();
                                spawn_tokio_future(tx, ctx.clone(), async move {
                                    let mut prof = profile;
                                    prof.instance.set_username(Username::new(username).unwrap());
                                    prof.instance.set_access_token(access_token);
                                    prof.instance.set_uuid(uuid);
                                    prof.launch()
                                        .await
                                        .map_err(|err| {
                                            let mut file =
                                                std::fs::File::create("./CRASH_REPORT.txt")
                                                    .expect("Cannot create CRASH_REPORT.txt");
                                            file.write_all(
                                                "App::download() paniced with following error: "
                                                    .as_bytes(),
                                            )
                                            .unwrap();
                                            file.write_all(format!("{:#?}", &err).as_bytes())
                                                .unwrap();
                                            err
                                        })
                                        .unwrap();
                                });
                            }
                        });
                    }
                });
            });
        });
    }
}

fn spawn_tokio_future<T, Fut>(
    tx: Sender<T>,
    ctx: egui::Context,
    fut: Fut,
) -> tokio::task::JoinHandle<()>
where
    T: 'static + Send,
    Fut: Future<Output = T> + Send + 'static,
{
    tokio::spawn(async move {
        let data = fut.await;
        let _ = tx.send(data);
        ctx.request_repaint();
    })
}

fn spawn_future<T, Fut>(tx: Sender<T>, ctx: egui::Context, fut: Fut) -> std::thread::JoinHandle<()>
where
    T: 'static + Send,
    Fut: Future<Output = T> + Send + 'static,
{
    std::thread::spawn(move || {
        let data = pollster::block_on(fut);
        let _ = tx.send(data);
        ctx.request_repaint();
    })
}
