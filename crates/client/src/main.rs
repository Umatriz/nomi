use context::AppContext;
use eframe::{
    egui::{self, ScrollArea, ViewportBuilder},
    epaint::Vec2,
};
use egui_tracing::EventCollector;
use std::fmt::Display;
use tracing::Level;
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
};
use utils::Crash;

pub mod client_settings;
pub mod context;
pub mod download;
pub mod utils;

fn main() {
    let collector = egui_tracing::EventCollector::default().with_level(Level::INFO);

    let appender = tracing_appender::rolling::hourly("./.nomi/logs", "nomi.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(appender);

    let mut file_sub = Layer::new()
        .with_writer(non_blocking.with_max_level(Level::INFO))
        .compact();
    file_sub.set_ansi(false);

    let mut stdout_sub = Layer::new()
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

    let _ = eframe::run_native(
        "Nomi",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(Vec2::new(1280.0, 720.0)),
            ..Default::default()
        },
        Box::new(|_cc| Box::new(AppTabs::new(collector))),
    );

    println!("T");
}

#[derive(PartialEq)]
pub enum Page {
    Main,
}

struct AppTabs {
    current: Page,
    profile_window: bool,
    settings_window: bool,
    logs_window: bool,

    context: AppContext,
}

impl AppTabs {
    pub fn new(collector: EventCollector) -> Self {
        Self {
            context: AppContext::new(collector).crash(),

            current: Page::Main,

            profile_window: Default::default(),
            settings_window: Default::default(),
            logs_window: Default::default(),
        }
    }
}

impl eframe::App for AppTabs {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_nav_bar").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                // ui.selectable_value(&mut self.current, Page::Main, "Main");
                ui.toggle_value(&mut self.profile_window, "Profile");

                // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.toggle_value(&mut self.settings_window, "Settings");

                ui.toggle_value(&mut self.logs_window, "Logs");
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

        egui::Window::new("Logs")
            .open(&mut self.logs_window)
            .resizable(true)
            .show(ctx, |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.add(egui_tracing::Logs::new(self.context.collector.clone()));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.current {
            Page::Main => self.context.show_main(ui),
        });
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
