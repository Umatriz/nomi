use crate::{
    download::spawn_download,
    utils::{spawn_future, spawn_tokio_future, Crash},
    Loader,
};
use eframe::egui::{self, Ui};
use egui_tracing::EventCollector;
use nomi_core::{
    configs::{
        profile::{VersionProfile, VersionProfilesConfig},
        read_toml_config_sync,
        user::Settings,
        write_toml_config_sync,
    },
    repository::{
        java_runner::JavaRunner, launcher_manifest::LauncherManifestVersion, username::Username,
    },
    utils::state::{launcher_manifest_state_try_init, LAUNCHER_MANIFEST_STATE},
};
use rfd::AsyncFileDialog;
use std::{
    io::Write,
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

pub struct AppContext {
    pub collector: EventCollector,

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
    pub fn new(collector: EventCollector) -> anyhow::Result<Self> {
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
            collector,
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
                let handle = spawn_download(
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
}