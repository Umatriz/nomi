use eframe::{
    egui::{self, Ui},
    epaint::Vec2,
};
use nomi_core::{
    configs::{
        profile::{VersionProfile, VersionProfileBuilder, VersionProfilesConfig},
        read_toml_config, read_toml_config_sync,
        user::Settings,
        write_toml_config, write_toml_config_sync,
    },
    instance::{launch::LaunchSettings, Inner, InstanceBuilder},
    repository::{
        java_runner::JavaRunner, launcher_manifest::LauncherManifestVersion, username::Username,
    },
    utils::state::{launcher_manifest_state_try_init, LAUNCHER_MANIFEST_STATE},
};
use rfd::AsyncFileDialog;
use std::{
    fmt::Display,
    future::Future,
    io::Write,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};
use tracing::Level;
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
    Registry,
};
use utils::Crash;

pub mod utils;

fn main() {
    let appender = tracing_appender::rolling::hourly("./.nomi/logs", "nomi.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);

    let mut file_sub = Layer::new()
        .with_writer(non_blocking.with_max_level(Level::INFO))
        .compact();
    file_sub.set_ansi(false);

    let mut stdout_sub = Layer::new()
        .with_writer(std::io::stdout.with_max_level(Level::INFO))
        .pretty();
    stdout_sub.set_ansi(false);

    let subscriber = Registry::default().with(stdout_sub).with(file_sub);

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
        eframe::NativeOptions {
            initial_window_size: Some(Vec2::new(1280.0, 720.0)),
            ..Default::default()
        },
        Box::new(|_cc| Box::new(AppTabs::new())),
    );

    println!("T");
}

struct AppTabs {
    current: Page,
    profile_window: bool,
    settings_window: bool,
    context: AppContext,
}

#[derive(PartialEq)]
pub enum Page {
    Main,
}

pub struct AppWindow<R> {
    name: &'static str,
    content: Box<dyn Fn(&mut Ui) -> R>,
}

impl<R> AppWindow<R> {
    pub fn new(name: &'static str, content: impl Fn(&mut Ui) -> R + 'static) -> Self {
        Self {
            name,
            content: Box::new(content),
        }
    }

    pub fn show(&self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name)
            .open(open)
            .show(ctx, |ui| (self.content)(ui));
    }
}

impl AppTabs {
    pub fn new() -> Self {
        Self {
            context: AppContext::new().crash(),

            current: Page::Main,
            profile_window: Default::default(),
            settings_window: Default::default(),
        }
    }
}

pub struct AppContext {
    tx: Sender<VersionProfile>,
    rx: Receiver<VersionProfile>,
    tasks: Vec<(tokio::task::JoinHandle<()>, String)>,

    profiles: VersionProfilesConfig,
    settings: Settings,
    // version_manifest: Option<&'static ManifestState>,
    release_versions: Option<Vec<&'static LauncherManifestVersion>>,

    profile_name_buf: String,
    selected_version_buf: usize,
    loader_buf: Loader,

    settings_username: Username,
    settings_username_buf: String,
    settings_uuid: String,
    settings_java_buf: JavaRunner,
    settings_java_buf_content: PathBuf,
    settings_java_buf_tx: Sender<JavaRunner>,
    settings_java_buf_rx: Receiver<JavaRunner>,
    settings_block_save_button: bool,
}

impl AppContext {
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        let (settings_java_buf_tx, settings_java_buf_rx) = std::sync::mpsc::channel();
        let profiles =
            read_toml_config_sync::<VersionProfilesConfig>("./.nomi/configs/Profiles.toml");
        let settings_res = read_toml_config_sync::<Settings>("./.nomi/configs/User.toml");
        let settings = settings_res.unwrap_or_default();

        let state = pollster::block_on(
            LAUNCHER_MANIFEST_STATE.get_or_try_init(launcher_manifest_state_try_init),
        );

        let java_bin = settings.java_bin.clone().unwrap_or_default();
        Ok(Self {
            release_versions: match state {
                Ok(data) => Some(
                    data.launcher
                        .versions
                        .iter()
                        .filter(|i| i.version_type == *"release")
                        .collect::<Vec<_>>(),
                ),
                Err(_) => None,
            },
            // version_manifest: match state {
            //     Ok(data) => Some(data),
            //     Err(_) => None,
            // },
            tx,
            rx,
            tasks: Default::default(),
            profiles: profiles.unwrap_or_default(),
            settings_username_buf: settings.username.get().to_string(),
            settings_java_buf_content: match &java_bin {
                JavaRunner::String(_) => PathBuf::new(),
                JavaRunner::Path(path) => path.to_path_buf(),
            },
            settings_java_buf: java_bin,
            settings,
            profile_name_buf: Default::default(),
            selected_version_buf: Default::default(),
            loader_buf: Loader::Vanilla,
            settings_java_buf_tx,
            settings_java_buf_rx,
            settings_uuid: "4350312f-04d5-4ee0-90b8-f883967593a0".to_string(),
            settings_block_save_button: false,
            settings_username: Default::default(),
        })
    }

    pub fn show_main(&mut self, ui: &mut Ui) {
        if let Ok(data) = self.rx.try_recv() {
            self.profiles.add_profile(data);
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
            egui::Grid::new("profiles_grid")
                .num_columns(3)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    for profile in self.profiles.profiles.clone() {
                        ui.label(profile.name.to_string());
                        ui.label(&profile.instance.settings.version);
                        if ui.button("Launch").clicked() {
                            let (tx, _rx) = std::sync::mpsc::channel();
                            let username = self.settings_username_buf.clone();
                            let access_token = self.settings.access_token.clone();
                            let uuid = self.settings.uuid.clone();
                            spawn_tokio_future(tx, async move {
                                let mut prof = profile;
                                prof.instance.set_username(Username::new(username).unwrap());
                                prof.instance.set_access_token(access_token);
                                prof.instance.set_uuid(uuid);
                                prof.launch()
                                    .await
                                    .map_err(|err| {
                                        let mut file = std::fs::File::create("./CRASH_REPORT.txt")
                                            .expect("Cannot create CRASH_REPORT.txt");
                                        file.write_all(
                                            "Nomi paniced with following error: ".as_bytes(),
                                        )
                                        .unwrap();
                                        file.write_all(format!("{:#?}", &err).as_bytes()).unwrap();
                                        err
                                    })
                                    .unwrap();
                            });
                        }
                        ui.end_row();
                    }
                });
        });
    }

    pub fn show_settings(&mut self, ui: &mut Ui) {
        if let Ok(data) = self.settings_java_buf_rx.try_recv() {
            self.settings_java_buf_content = data.get().into();
            self.settings_java_buf = data;
        }
        ui.collapsing("User", |ui| {
            ui.label(
                egui::RichText::new("This category will be replaced with Microsoft Auth")
                    .font(egui::FontId::proportional(20.0)),
            );
            ui.label("Username");
            ui.text_edit_singleline(&mut self.settings_username_buf);
            match Username::new(self.settings_username_buf.clone()) {
                Err(err) => {
                    ui.label(format!("{}", err));
                    self.settings_block_save_button = true;
                }
                Ok(data) => {
                    self.settings_username = data;
                    self.settings_block_save_button = false;
                }
            }
            ui.label("uuid");
            ui.text_edit_singleline(&mut self.settings_uuid)
                .on_hover_text("By default is just a random uuid (hardcoded).");
        });
        ui.collapsing("Java", |ui| {
            ui.label(egui::RichText::new("For legacy versions such 1.0, 1.2 etc you should specify java 8 binary").font(egui::FontId::proportional(16.0)));
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut self.settings_java_buf,
                    JavaRunner::Path(self.settings_java_buf_content.clone()),
                    "Path",
                ).on_hover_text("Set path directly to your java bin file.");
                ui.radio_value(
                    &mut self.settings_java_buf,
                    JavaRunner::default(),
                    "Command",
                )
                .on_hover_text("All command will be execute using `java` command. You must have java in your PATH.");
            });
            if let JavaRunner::Path(_) = self.settings_java_buf {
                ui.label(self.settings_java_buf.get_string());
                if ui.button("Select java").clicked() {
                    spawn_future(self.settings_java_buf_tx.clone(), async {
                        let file = AsyncFileDialog::new()
                            .add_filter("bin", &["exe"])
                            .set_directory("/")
                            .pick_file()
                            .await;

                        let Some(binding) = file else {
                            return JavaRunner::default();
                        };
                        let path = binding.path();

                        JavaRunner::path(path.to_path_buf())
                    });
                };
            }
        });
        match self.settings_block_save_button {
            true => {
                ui.add_enabled(false, egui::Button::new("Save"));
            }
            false => {
                if ui.button("Save").clicked() {
                    let settings = Settings {
                        username: Username::new(self.settings_username_buf.clone()).crash(),
                        access_token: None,
                        java_bin: Some(self.settings_java_buf.clone()),
                        uuid: Some(self.settings_uuid.clone()),
                    };
                    let _ = write_toml_config_sync(&settings, "./.nomi/configs/User.toml");
                }
            }
        }
    }

    pub fn show_profiles(&mut self, ui: &mut Ui) {
        if let Some(profiles) = &self.release_versions {
            ui.label("Create new profile");
            ui.label("Profile name:");
            ui.text_edit_singleline(&mut self.profile_name_buf);
            egui::ComboBox::from_label("Select version")
                .selected_text(&profiles[self.selected_version_buf].id)
                .show_ui(ui, |ui| {
                    for i in 0..profiles.len() {
                        let value = ui.selectable_value(
                            &mut &profiles[i],
                            &profiles[self.selected_version_buf],
                            &profiles[i].id,
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
                    // ctx.clone(),
                    self.profile_name_buf.clone(),
                    profiles[self.selected_version_buf].id.clone(),
                    self.loader_buf.clone(),
                );
                self.tasks.push((
                    handle,
                    format!(
                        "Downloading version {} - {}",
                        profiles[self.selected_version_buf].id, self.loader_buf
                    ),
                ))
            }
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
    }

    pub fn spawn_download(
        tx: Sender<VersionProfile>,
        name: String,
        version: String,
        loader: Loader,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let data = Self::try_download(name, version, loader).await.crash();
            let _ = tx.send(data);
        })
    }

    async fn try_download(
        name: String,
        version: String,
        loader: Loader,
    ) -> anyhow::Result<VersionProfile> {
        // return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Error").into());
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

impl eframe::App for AppTabs {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.2);
        egui::TopBottomPanel::top("top_nav_bar").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                // ui.selectable_value(&mut self.current, Page::Main, "Main");
                ui.toggle_value(&mut self.settings_window, "Settings");

                // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.toggle_value(&mut self.profile_window, "Profile");
                // });
            });
        });

        egui::Window::new("Profile")
            .open(&mut self.profile_window)
            .resizable(false)
            .show(ctx, |ui| {
                self.context.show_profiles(ui);
            });

        egui::Window::new("Settings")
            .open(&mut self.settings_window)
            .resizable(false)
            .show(ctx, |ui| {
                self.context.show_settings(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.current {
            Page::Main => self.context.show_main(ui),
        });
    }
}

fn spawn_tokio_future<T, Fut>(tx: Sender<T>, fut: Fut) -> tokio::task::JoinHandle<()>
where
    T: 'static + Send,
    Fut: Future<Output = T> + Send + 'static,
{
    tokio::spawn(async move {
        let data = fut.await;
        let _ = tx.send(data);
    })
}

fn spawn_future<T, Fut>(tx: Sender<T>, fut: Fut) -> std::thread::JoinHandle<()>
where
    T: 'static + Send,
    Fut: Future<Output = T> + Send + 'static,
{
    std::thread::spawn(move || {
        let data = pollster::block_on(fut);
        let _ = tx.send(data);
    })
}
