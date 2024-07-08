use collections::{AssetsCollection, GameDownloadingCollection, JavaCollection};
use context::MyContext;
use eframe::{
    egui::{self, Align, Align2, Frame, Id, Layout, RichText, ViewportBuilder},
    epaint::Vec2,
};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use egui_tracing::EventCollector;
use views::{add_tab_menu::AddTab, View};

use errors_pool::ERRORS_POOL;
use nomi_core::DOT_NOMI_LOGS_DIR;
use tracing::{info, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
    EnvFilter,
};

pub mod download;
pub mod errors_pool;
pub mod utils;
pub mod views;

pub mod simplify;

pub mod collections;

pub mod popup;
pub mod tab;
pub use tab::*;
pub mod context;
pub mod states;

fn main() {
    let collector = egui_tracing::EventCollector::default().with_level(Level::INFO);

    let appender = tracing_appender::rolling::hourly(DOT_NOMI_LOGS_DIR, "nomi.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);

    let mut file_sub = Layer::new()
        .with_writer(non_blocking.with_max_level(Level::INFO))
        .compact();
    file_sub.set_ansi(false);

    let stdout_sub = Layer::new()
        .with_writer(std::io::stdout.with_max_level(Level::DEBUG))
        .pretty();
    // stdout_sub.set_ansi(false);

    let subscriber = tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .parse("client=debug,nomi_core=debug")
                .unwrap(),
        )
        .with(collector.clone())
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
        Box::new(|_cc| Ok(Box::new(MyTabs::new(collector)))),
    );

    info!("Exiting")
}

struct MyTabs {
    context: MyContext,
    dock_state: DockState<Tab>,
}

impl MyTabs {
    pub fn new(collector: EventCollector) -> Self {
        let tabs = vec![
            Tab::from_tab_kind(TabKind::Profiles),
            Tab::from_tab_kind(TabKind::Logs),
            Tab::from_tab_kind(TabKind::Settings),
        ];

        let mut dock_state = DockState::new(tabs);

        let surface = dock_state.main_surface_mut();
        surface.split_right(
            NodeIndex::root(),
            0.60,
            vec![Tab::from_tab_kind(TabKind::DownloadProgress)],
        );

        Self {
            context: MyContext::new(collector),
            dock_state,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let mut style = Style::from_egui(ui.style().as_ref());

        style.tab.tab_body.stroke.width = 0.0;

        DockArea::new(&mut self.dock_state)
            .style(style)
            .show_inside(ui, &mut self.context);
    }
}

impl eframe::App for MyTabs {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.context
            .manager
            .add_collection::<collections::AssetsCollection>(())
            .add_collection::<collections::FabricDataCollection>(
                &mut self.context.states.add_profile_menu_state.fabric_versions,
            )
            .add_collection::<collections::GameDeletionCollection>(())
            .add_collection::<collections::GameDownloadingCollection>(
                &mut self.context.states.profiles.profiles,
            )
            .add_collection::<collections::JavaCollection>(());

        ctx.set_pixels_per_point(self.context.states.client_settings.pixels_per_point);

        if !self.context.states.java.is_downloaded {
            self.context
                .states
                .java
                .download_java(&mut self.context.manager);
        }

        egui::TopBottomPanel::top("top_panel_id").show(ctx, |ui| {
            ui.with_layout(
                Layout::left_to_right(Align::Center).with_cross_align(Align::Center),
                |ui| {
                    // The way to calculate the target size is captured from the
                    // https://github.com/emilk/egui/discussions/3908 big thanks ^_^
                    let id_cal_target_size = Id::new("cal_target_size");
                    let this_init_max_width = ui.max_rect().width();
                    let last_others_width = ui.data(|data| {
                        data.get_temp(id_cal_target_size)
                            .unwrap_or(this_init_max_width)
                    });
                    let this_target_width = this_init_max_width - last_others_width;

                    AddTab {
                        dock_state: &self.dock_state,
                        tabs_state: &mut self.context.states.tabs,
                    }
                    .ui(ui);

                    ui.add_space(this_target_width);
                    ui.horizontal(|ui| {
                        egui::warn_if_debug_build(ui);
                        ui.hyperlink_to(
                            RichText::new(format!(
                                "{} Nomi on GitHub",
                                egui::special_emojis::GITHUB
                            ))
                            .small(),
                            "https://github.com/Umatriz/nomi",
                        );
                        ui.hyperlink_to(
                            RichText::new("Nomi's Discord server").small(),
                            "https://discord.gg/qRD5XEJKc4",
                        );
                    });

                    ui.data_mut(|data| {
                        data.insert_temp(
                            id_cal_target_size,
                            ui.min_rect().width() - this_target_width,
                        )
                    });
                },
            );
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
                            for error in pool.iter_errors() {
                                ui.label(format!("{}", error));
                            }
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

        let opened_tabs = self
            .dock_state
            .iter_all_tabs()
            .map(|(_, tab)| tab.id().clone())
            .collect::<Vec<_>>();

        for tab_id in &self.context.states.tabs.0 {
            if !opened_tabs.contains(tab_id) {
                self.dock_state
                    .push_to_first_leaf(Tab::from_tab_kind(TabKind::from_id(tab_id.to_owned())))
            }
        }

        for tab_id in &opened_tabs {
            if !self.context.states.tabs.0.contains(tab_id) {
                self.dock_state
                    .find_tab(&Tab::from_tab_kind(TabKind::from_id(tab_id.clone())))
                    .and_then(|tab_info| self.dock_state.remove_tab(tab_info));
            }
        }

        let manager = &self.context.manager;

        self.context.is_allowed_to_take_action = [
            manager.get_collection::<AssetsCollection>(),
            manager.get_collection::<JavaCollection>(),
            manager.get_collection::<GameDownloadingCollection>(),
        ]
        .iter()
        .all(|c| c.tasks().is_empty());
    }
}
