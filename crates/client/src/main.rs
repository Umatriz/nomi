use anyhow::anyhow;
use components::{add_profile_menu::AddProfileMenuState, add_tab_menu::AddTab, Component};
use context::MyContext;
use eframe::{
    egui::{self, Align, Align2, Frame, Layout, ViewportBuilder},
    epaint::Vec2,
};
use egui_dock::{DockArea, DockState, Style};
use egui_tracing::EventCollector;

use errors_pool::ERRORS_POOL;
use tracing::{info, Level};
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
};

pub mod components;
pub mod download;
pub mod errors_pool;
pub mod utils;

pub mod channel;
pub mod tab;
pub use tab::*;
pub mod context;
pub mod states;

fn main() {
    let collector = egui_tracing::EventCollector::default().with_level(Level::INFO);

    let appender = tracing_appender::rolling::hourly("./.nomi/logs", "nomi.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);

    let mut file_sub = Layer::new()
        .with_writer(non_blocking.with_max_level(Level::INFO))
        .compact();
    file_sub.set_ansi(false);

    let stdout_sub = Layer::new()
        .with_writer(std::io::stdout.with_max_level(Level::INFO))
        .pretty();
    // stdout_sub.set_ansi(false);

    let subscriber = tracing_subscriber::registry()
        .with(collector.clone())
        .with(stdout_sub)
        .with(file_sub);

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

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(1280.0, 720.0)),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Nomi",
        native_options,
        Box::new(|_cc| Box::new(MyTabs::new(collector))),
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
            Tab::from_tab_kind(TabKind::Profiles {
                menu_state: AddProfileMenuState::default(),
            }),
            Tab::from_tab_kind(TabKind::Settings),
        ];

        let dock_state = DockState::new(tabs);

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
        ctx.set_pixels_per_point(self.context.states.client_settings.pixels_per_point);

        egui::TopBottomPanel::top("top_panel_id").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                AddTab {
                    dock_state: &self.dock_state,
                    tabs_state: &mut self.context.states.tabs,
                }
                .ui(ui);
                egui::warn_if_debug_build(ui);
                if ui.button("Make an error!").clicked() {
                    ERRORS_POOL.write().unwrap().push_error(anyhow!("Error!"));
                }
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
    }
}
