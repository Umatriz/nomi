// Remove console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use collections::{AssetsCollection, GameDownloadingCollection, GameRunnerCollection, JavaCollection};
use context::MyContext;
use eframe::{
    egui::{self, Align, Align2, Button, Frame, Id, Layout, RichText, ScrollArea, ViewportBuilder},
    epaint::Vec2,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use egui_notify::Toasts;
use open_directory::open_directory_native;
use std::path::Path;
use subscriber::EguiLayer;
use ui_ext::TOASTS_ID;
use views::{add_tab_menu::AddTab, View};

use errors_pool::{ErrorPoolExt, ERRORS_POOL};
use nomi_core::{DOT_NOMI_DATA_PACKS_DIR, DOT_NOMI_LOGS_DIR};
use tracing::{info, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
    EnvFilter,
};

pub mod consts;
pub mod download;
pub mod errors_pool;
pub mod ui_ext;
pub mod utils;
pub mod views;

pub mod mods;
pub mod open_directory;

pub mod collections;
pub mod progress;
pub mod subscriber;

pub mod tab;
pub use tab::*;
pub mod context;
pub mod states;

pub use consts::*;

fn main() {
    let appender = tracing_appender::rolling::hourly(DOT_NOMI_LOGS_DIR, "nomi.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);

    let mut file_sub = Layer::new().with_writer(non_blocking.with_max_level(Level::INFO)).compact();
    file_sub.set_ansi(false);

    let stdout_sub = Layer::new().with_writer(std::io::stdout.with_max_level(Level::DEBUG)).pretty();
    // stdout_sub.set_ansi(false);

    let egui_layer = EguiLayer::new().with_level(Level::DEBUG);

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::builder().parse("client=debug,nomi_core=debug").unwrap())
        .with(egui_layer.clone())
        .with(stdout_sub)
        .with(file_sub);

    tracing::subscriber::set_global_default(subscriber).unwrap();

    egui_task_manager::setup!();

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(1280.0, 720.0)),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Nomi",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyTabs::new(egui_layer)))
        }),
    );

    info!("Exiting")
}

struct MyTabs {
    context: MyContext,
    dock_state: DockState<Tab>,
}

impl MyTabs {
    pub fn new(egui_layer: EguiLayer) -> Self {
        let tabs = [TabKind::Profiles, TabKind::Logs, TabKind::Settings]
            .map(|kind| Tab { id: kind.id(), kind })
            .into();

        let mut dock_state = DockState::new(tabs);

        let surface = dock_state.main_surface_mut();
        surface.split_right(
            NodeIndex::root(),
            0.60,
            vec![Tab {
                id: TabKind::DownloadProgress.id(),
                kind: TabKind::DownloadProgress,
            }],
        );

        Self {
            context: MyContext::new(egui_layer),
            dock_state,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut style = Style::from_egui(ui.style().as_ref());

        style.tab.tab_body.stroke.width = 0.0;

        DockArea::new(&mut self.dock_state).style(style).show_inside(ui, &mut self.context);
    }
}

impl eframe::App for MyTabs {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            use parking_lot::Mutex;
            use std::sync::Arc;

            let toasts = ctx.data_mut(|data| data.get_temp_mut_or_default::<Arc<Mutex<Toasts>>>(egui::Id::new(TOASTS_ID)).clone());

            let mut locked = toasts.lock();

            locked.show(ctx);
        }

        self.context
            .manager
            .add_collection::<collections::AssetsCollection>(())
            .add_collection::<collections::FabricDataCollection>(&mut self.context.states.add_profile_menu_state.fabric_versions)
            .add_collection::<collections::GameDeletionCollection>(())
            .add_collection::<collections::GameDownloadingCollection>(&self.context.states.profiles.instances)
            .add_collection::<collections::JavaCollection>(())
            .add_collection::<collections::ProjectCollection>(&mut self.context.states.mod_manager.current_project)
            .add_collection::<collections::ProjectVersionsCollection>(&mut self.context.states.mod_manager.current_versions)
            .add_collection::<collections::DependenciesCollection>((
                &mut self.context.states.mod_manager.current_dependencies,
                self.context.states.mod_manager.current_project.as_ref().map(|p| &p.id),
            ))
            .add_collection::<collections::ModsDownloadingCollection>(&self.context.states.profiles.instances)
            .add_collection::<collections::GameRunnerCollection>(())
            .add_collection::<collections::DownloadAddedModsCollection>((
                &mut self.context.states.profile_info.currently_downloading_mods,
                &self.context.states.profiles.instances,
            ));

        ctx.set_pixels_per_point(self.context.states.client_settings.pixels_per_point);

        if !self.context.states.java.is_downloaded {
            self.context.states.java.download_java(&mut self.context.manager);
        }

        egui::TopBottomPanel::top("top_panel_id").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center).with_cross_align(Align::Center), |ui| {
                // The way to calculate the target size is captured from the
                // https://github.com/emilk/egui/discussions/3908 big thanks ^_^
                let id_cal_target_size = Id::new("cal_target_size");
                let this_init_max_width = ui.max_rect().width();
                let last_others_width = ui.data(|data| data.get_temp(id_cal_target_size).unwrap_or(this_init_max_width));
                let this_target_width = this_init_max_width - last_others_width;

                AddTab {
                    dock_state: &self.dock_state,
                    tabs_state: &mut self.context.states.tabs,
                }
                .ui(ui);

                ui.menu_button("Open", |ui| {
                    if ui
                        .add_enabled(Path::new(DOT_NOMI_DATA_PACKS_DIR).exists(), Button::new("Data Packs"))
                        .on_disabled_hover_text("You did not downloaded any datapacks.")
                        .clicked()
                    {
                        if let Ok(path) = std::fs::canonicalize(DOT_NOMI_DATA_PACKS_DIR) {
                            open_directory_native(path).report_error();
                        }
                    }
                });

                ui.add_space(this_target_width);
                ui.horizontal(|ui| {
                    egui::warn_if_debug_build(ui);
                    ui.hyperlink_to(
                        RichText::new(format!("{} Nomi on GitHub", egui::special_emojis::GITHUB)).small(),
                        "https://github.com/Umatriz/nomi",
                    );
                    ui.hyperlink_to(RichText::new("Nomi's Discord server").small(), "https://discord.gg/qRD5XEJKc4");
                });

                ui.data_mut(|data| data.insert_temp(id_cal_target_size, ui.min_rect().width() - this_target_width));
            });
        });

        if let Ok(len) = ERRORS_POOL.try_read().map(|pool| pool.len()) {
            if self.context.states.errors_pool.number_of_errors != len {
                self.context.states.errors_pool.number_of_errors = len;
                self.context.states.errors_pool.is_window_open = true;
            }
        }

        egui::Window::new("Errors")
            .id("error_window".into())
            .open(&mut self.context.states.errors_pool.is_window_open)
            .resizable(false)
            .movable(false)
            .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
            .show(ctx, |ui| {
                {
                    match ERRORS_POOL.try_read() {
                        Ok(pool) => {
                            if pool.is_empty() {
                                ui.label("No errors");
                            }
                            ScrollArea::vertical().show(ui, |ui| {
                                ui.vertical(|ui| {
                                    for error in pool.iter_errors() {
                                        ui.label(format!("{:#?}", error));
                                        ui.separator();
                                    }
                                });
                            });
                        }
                        Err(_) => {
                            ui.spinner();
                        }
                    }
                }

                if ui.button("Clear").clicked() {
                    ERRORS_POOL.write().unwrap().clear()
                }
            });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(ctx.style().as_ref()).inner_margin(0.0))
            .show(ctx, |ui| self.ui(ui));

        let opened_tabs = self.dock_state.iter_all_tabs().map(|(_, tab)| tab.id.clone()).collect::<Vec<_>>();

        for (id, kind) in &self.context.states.tabs.0 {
            if !opened_tabs.contains(id) {
                self.dock_state.push_to_first_leaf(Tab {
                    id: id.clone(),
                    kind: kind.clone(),
                })
            }
        }

        for id in &opened_tabs {
            if !self.context.states.tabs.0.contains_key(id) {
                // `TabKind` here does not matter since when comparing `Tab`s you're comparing their `id`s
                self.dock_state
                    .find_tab(&Tab {
                        id: id.clone(),
                        kind: TabKind::Logs,
                    })
                    .and_then(|tab_info| self.dock_state.remove_tab(tab_info));
            }
        }

        if self.context.images_clean_requested {
            ctx.forget_all_images();
            self.context.images_clean_requested = false
        }

        let manager = &self.context.manager;

        self.context.is_allowed_to_take_action = [
            manager.get_collection::<AssetsCollection>(),
            manager.get_collection::<JavaCollection>(),
            manager.get_collection::<GameDownloadingCollection>(),
            manager.get_collection::<GameRunnerCollection>(),
        ]
        .iter()
        .all(|c| c.tasks().is_empty());
    }
}
