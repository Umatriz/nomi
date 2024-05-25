use std::{path::PathBuf, str::FromStr};

use eframe::egui;
use egui_file_dialog::FileDialog;
use egui_form::{garde::field_path, Form, FormField};
use garde::{Error, Validate};
use nomi_core::{
    configs::user::Settings,
    fs::write_toml_config_sync,
    regex::Regex,
    repository::{java_runner::JavaRunner, username::Username},
    Uuid,
};

use crate::Storage;

use super::{Component, StorageCreationExt};

pub struct SettingsPage<'a> {
    pub storage: &'a mut Storage,
    pub file_dialog: &'a mut FileDialog,
}

#[derive(Debug, Validate)]
pub(crate) struct SettingsData {
    #[garde(custom(check_username))]
    username: String,
    #[garde(custom(check_uuid))]
    uuid: String,
    #[garde(skip)]
    java: JavaRunner,
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
        let uuid = Uuid::new_v4().to_string();
        storage.insert(SettingsData {
            username: "Nomi".to_owned(),
            uuid: uuid.clone(),
            java: JavaRunner::command("java"),
        });

        write_toml_config_sync(
            &Settings {
                username: Username::new("Nomi").unwrap(),
                access_token: None,
                java_bin: Some(JavaRunner::Command("java".to_owned())),
                uuid: Some(uuid),
            },
            "./.nomi/configs/Settings.toml",
        )
        .unwrap();

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

        FormField::new(&mut form, field_path!("username"))
            .label("Username")
            .ui(ui, egui::TextEdit::singleline(&mut settings_data.username));

        FormField::new(&mut form, field_path!("uuid"))
            .label("UUID")
            .ui(ui, egui::TextEdit::singleline(&mut settings_data.uuid));

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

        if let Some(Ok(())) = form.handle_submit(&ui.button("Save"), ui) {
            write_toml_config_sync(
                &Settings {
                    username: Username::new(&settings_data.username).unwrap(),
                    access_token: None,
                    java_bin: Some(settings_data.java.clone()),
                    uuid: Some(settings_data.uuid.to_owned()),
                },
                "./.nomi/configs/Settings.toml",
            )
            .unwrap();
        }
    }
}
