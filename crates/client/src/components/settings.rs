use std::path::PathBuf;

use eframe::egui::{self, RichText};
use egui_file_dialog::FileDialog;
use egui_form::{garde::field_path, Form, FormField};
use garde::{Error, Validate};
use nomi_core::{
    fs::write_toml_config_sync, regex::Regex, repository::java_runner::JavaRunner, Uuid,
};
use serde::{Deserialize, Serialize};

use crate::{download_java, errors_pool::ErrorPoolExt, states::JavaState};

use super::{download_progress::DownloadProgressState, Component};

pub struct SettingsPage<'a> {
    pub java_state: &'a mut JavaState,
    pub download_progress_state: &'a mut DownloadProgressState,

    pub settings_state: &'a mut SettingsState,
    pub client_settings_state: &'a mut ClientSettingsState,
    pub file_dialog: &'a mut FileDialog,
}

#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
pub struct SettingsState {
    #[garde(custom(check_username))]
    pub username: String,
    #[garde(custom(check_uuid))]
    pub uuid: String,
    #[garde(skip)]
    pub java: JavaRunner,

    #[garde(skip)]
    pub client_settings: ClientSettingsState,
}

impl Default for SettingsState {
    fn default() -> Self {
        SettingsState {
            username: "Nomi".to_owned(),
            uuid: Uuid::new_v4().to_string(),
            java: JavaRunner::command("java"),
            client_settings: ClientSettingsState::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientSettingsState {
    pub pixels_per_point: f32,
}

impl Default for ClientSettingsState {
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

impl Component for SettingsPage<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        ui.collapsing("Utils", |ui| {
            let launcher_path = PathBuf::from("./.nomi/logs");

            if launcher_path.exists() {
                if ui.button("Delete launcher's logs").clicked() {
                    let _ = std::fs::remove_dir_all(launcher_path);
                }
            } else {
                ui.label(
                    RichText::new("The launcher log's directory is already deleted")
                        .color(ui.visuals().warn_fg_color),
                );
            }

            let game_path = PathBuf::from("./logs");

            if game_path.exists() {
                if ui.button("Delete game's logs").clicked() {
                    let _ = std::fs::remove_dir_all(game_path);
                }
            } else {
                ui.label(
                    RichText::new("The games log's directory is already deleted")
                        .color(ui.visuals().warn_fg_color),
                );
            }
        });

        let settings_data = self.settings_state.clone();

        let mut form = Form::new().add_report(egui_form::garde::GardeReport::new(
            settings_data.validate(&()),
        ));

        {
            if let Some(path) = self.file_dialog.update(ui.ctx()).selected() {
                if let JavaRunner::Path(java_path) = &mut self.settings_state.java {
                    if java_path != path {
                        *java_path = dbg!(path).to_path_buf();
                    }
                }
            }

            ui.collapsing("User", |ui| {
                FormField::new(&mut form, field_path!("username"))
                    .label("Username")
                    .ui(
                        ui,
                        egui::TextEdit::singleline(&mut self.settings_state.username),
                    );

                FormField::new(&mut form, field_path!("uuid"))
                    .label("UUID")
                    .ui(
                        ui,
                        egui::TextEdit::singleline(&mut self.settings_state.uuid),
                    );
            });

            ui.collapsing("Java", |ui| {
                if ui.add_enabled(self.download_progress_state.java_downloading_task.is_none(), egui::Button::new("Download Java")).on_hover_text("Pressing this button will start the Java downloading process and add the downloaded binary as selected").clicked() {
                    download_java(self.java_state, self.download_progress_state);
                    // self.settings_state.java = JavaRunner::path(PathBuf::from(value))
                }
                FormField::new(&mut form, field_path!("java"))
                    .label("Java")
                    .ui(ui, |ui: &mut egui::Ui| {
                        ui.radio_value(
                            &mut self.settings_state.java,
                            JavaRunner::command("java"),
                            "Command",
                        );

                        ui.radio_value(
                            &mut self.settings_state.java,
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
                        &mut self.settings_state.client_settings.pixels_per_point,
                        0.5..=5.0,
                    )
                    .text("Pixels per point"),
                )
            });
        }

        if let Some(Ok(())) = form.handle_submit(&ui.button("Save"), ui) {
            *self.client_settings_state = settings_data.client_settings.clone();
            write_toml_config_sync(&settings_data, "./.nomi/configs/Settings.toml").report_error();
        }
    }
}
