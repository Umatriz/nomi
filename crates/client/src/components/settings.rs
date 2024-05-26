use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::FileDialog;
use egui_form::{garde::field_path, Form, FormField};
use garde::{Error, Validate};
use nomi_core::{
    fs::{read_toml_config_sync, write_toml_config_sync},
    regex::Regex,
    repository::java_runner::JavaRunner,
    Uuid,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::Storage;

use super::{Component, StorageCreationExt};

pub struct SettingsPage<'a> {
    pub storage: &'a mut Storage,
    pub file_dialog: &'a mut FileDialog,
}

#[derive(Debug, Validate, Serialize, Deserialize)]
pub(crate) struct SettingsData {
    #[garde(custom(check_username))]
    username: String,
    #[garde(custom(check_uuid))]
    uuid: String,
    #[garde(skip)]
    java: JavaRunner,

    #[garde(skip)]
    client_settings: ClientSettings,
}

impl Default for SettingsData {
    fn default() -> Self {
        SettingsData {
            username: "Nomi".to_owned(),
            uuid: Uuid::new_v4().to_string(),
            java: JavaRunner::command("java"),
            client_settings: ClientSettings::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ClientSettings {
    pixels_per_point: f32,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            pixels_per_point: 1.5,
        }
    }
}

fn check_username(value: &str, _context: &()) -> garde::Result {
    let regex = Regex::new(r"^[a-zA-Z0-9_]{3,16}$").map_err(|_| {
        Error::new("Cannot create regex (this is a bug, please create an issue on the github)")
    })?;

    regex.captures(value).map_or_else(
        || {
            Err(Error::new(
                "
Invalid username form
The username cannot be more than 16 letters or less than 3
You may use:
A-Z characters, a-z characters, 0-9 numbers, `_` (underscore) symbol
    ",
            ))
        },
        |_| Ok(()),
    )
}

fn check_uuid(value: &str, _context: &()) -> garde::Result {
    let regex = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
        .map_err(|_| {
            Error::new("Cannot create regex (this is a bug, please create an issue on the github)")
        })?;

    regex
        .captures(value)
        .map_or_else(|| Err(Error::new("Invalid UUID")), |_| Ok(()))
}

impl StorageCreationExt for SettingsPage<'_> {
    fn extend(storage: &mut Storage) -> anyhow::Result<()> {
        let data = match read_toml_config_sync::<SettingsData>("./.nomi/configs/Settings.toml") {
            Ok(data) => data,
            Err(e) => {
                error!("{}", e);
                write_toml_config_sync(&SettingsData::default(), "./.nomi/configs/Settings.toml")?;
                SettingsData::default()
            }
        };

        storage.insert(data);

        Ok(())
    }
}

impl Component for SettingsPage<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        let settings_data = self.storage.get_mut::<SettingsData>().unwrap();

        let mut form = Form::new().add_report(egui_form::garde::GardeReport::new(
            settings_data.validate(&()),
        ));

        if let Some(path) = self.file_dialog.update(ui.ctx()).selected() {
            if let JavaRunner::Path(java_path) = &mut settings_data.java {
                if java_path != path {
                    *java_path = dbg!(path).to_path_buf();
                }
            }
        }

        ui.collapsing("User", |ui| {
            FormField::new(&mut form, field_path!("username"))
                .label("Username")
                .ui(ui, egui::TextEdit::singleline(&mut settings_data.username));

            FormField::new(&mut form, field_path!("uuid"))
                .label("UUID")
                .ui(ui, egui::TextEdit::singleline(&mut settings_data.uuid));
        });

        ui.collapsing("Java", |ui| {
            FormField::new(&mut form, field_path!("java"))
                .label("Java")
                .ui(ui, |ui: &mut egui::Ui| {
                    ui.radio_value(
                        &mut settings_data.java,
                        JavaRunner::command("java"),
                        "Command",
                    );

                    ui.radio_value(
                        &mut settings_data.java,
                        JavaRunner::path(PathBuf::new()),
                        "Custom path",
                    );

                    if matches!(settings_data.java, JavaRunner::Path(_))
                        && ui.button("Select custom java binary").clicked()
                    {
                        self.file_dialog.select_file();
                    }

                    ui.label(format!(
                        "Java will be run using {}",
                        match &settings_data.java {
                            JavaRunner::Command(command) => format!("{} command", command),
                            JavaRunner::Path(path) => format!("{} executable", path.display()),
                        }
                    ))
                });
        });

        ui.collapsing("Client", |ui| {
            ui.add(
                egui::Slider::new(
                    &mut settings_data.client_settings.pixels_per_point,
                    0.5..=5.0,
                )
                .text("Pixels per point"),
            )
        });

        if let Some(Ok(())) = form.handle_submit(&ui.button("Save"), ui) {
            write_toml_config_sync(&settings_data, "./.nomi/configs/Settings.toml").unwrap();
        }
    }
}
