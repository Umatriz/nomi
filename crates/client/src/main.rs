use components::{
    add_profile_menu::{AddProfileMenu, AddProfileMenuState},
    add_tab_menu::{AddTab, TabsState},
    download_progress::DownloadProgress,
    profiles::{ProfilesPage, ProfilesState},
    settings::{ClientSettings, SettingsPage},
    Component, StorageCreationExt,
};
use eframe::{
    egui::{self, Align, Frame, Layout, ScrollArea, ViewportBuilder},
    epaint::Vec2,
};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use egui_file_dialog::FileDialog;
use egui_tracing::EventCollector;
use nomi_core::{
    configs::profile::VersionProfile, downloads::traits::DownloadResult,
    repository::launcher_manifest::LauncherManifest, state::get_launcher_manifest,
};
use std::{collections::HashSet, ops::Deref};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::Level;
use tracing_subscriber::{
    fmt::{writer::MakeWriterExt, Layer},
    prelude::__tracing_subscriber_SubscriberExt,
};
use type_map::TypeMap;

pub mod components;
pub mod download;
pub mod type_map;
pub mod utils;

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

    let _ = eframe::run_native(
        "Nomi",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size(Vec2::new(1280.0, 720.0)),
            ..Default::default()
        },
        Box::new(|_cc| Box::new(MyTabs::new(collector))),
    );

    println!("T");
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct TabId(&'static str);

impl TabId {
    pub const PROFILES: Self = Self("Profiles");
    pub const SETTINGS: Self = Self("Settings");
    pub const LOGS: Self = Self("Logs");
    pub const DOWNLOAD_STATUS: Self = Self("Download Status");
}

pub enum Tab {
    Profiles {
        profiles_state: ProfilesState,
        menu_state: AddProfileMenuState,
    },
    Settings,
    Logs,
    DownloadStatus,
}

impl PartialEq for Tab {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl Tab {
    pub const AVAILABLE_TABS: &'static [Tab] = &[
        Self::Profiles {
            profiles_state: ProfilesState::default_const(),
            menu_state: AddProfileMenuState::default_const(),
        },
        Self::Settings,
        Self::Logs,
        Self::DownloadStatus,
    ];

    pub fn from_id(id: TabId) -> Self {
        match id {
            TabId::PROFILES => Tab::Profiles {
                profiles_state: Default::default(),
                menu_state: Default::default(),
            },
            TabId::SETTINGS => Tab::Settings,
            TabId::LOGS => Tab::Logs,
            TabId::DOWNLOAD_STATUS => Tab::DownloadStatus,
            _ => unreachable!(),
        }
    }

    pub fn id(&self) -> TabId {
        match self {
            Tab::Profiles { .. } => TabId::PROFILES,
            Tab::Settings => TabId::SETTINGS,
            Tab::Logs => TabId::LOGS,
            Tab::DownloadStatus => TabId::DOWNLOAD_STATUS,
        }
    }

    pub fn name(&self) -> &'static str {
        self.id().0
    }
}

pub type Storage = TypeMap;

pub struct Channel<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer);
        Self { tx, rx }
    }

    pub fn clone_tx(&self) -> Sender<T> {
        self.tx.clone()
    }
}

impl<T> Deref for Channel<T> {
    type Target = Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.rx
    }
}

struct MyContext {
    collector: EventCollector,
    storage: Storage,
    launcher_manifest: &'static LauncherManifest,

    tabs_state: TabsState,

    file_dialog: FileDialog,

    is_profile_window_open: bool,

    download_result_channel: Channel<VersionProfile>,
    download_progress_channel: Channel<DownloadResult>,
    download_total_channel: Channel<u32>,
}

impl MyContext {
    pub fn new(collector: EventCollector) -> Self {
        let launcher_manifest_ref = pollster::block_on(get_launcher_manifest()).unwrap();

        let mut storage = Storage::new();

        // TODO: handle errors properly
        ProfilesPage::extend(&mut storage).unwrap();
        AddProfileMenu::extend(&mut storage).unwrap();
        DownloadProgress::extend(&mut storage).unwrap();
        SettingsPage::extend(&mut storage).unwrap();

        let mut tabs = HashSet::new();

        tabs.insert(TabId::PROFILES);
        tabs.insert(TabId::SETTINGS);

        Self {
            storage,
            collector,
            launcher_manifest: launcher_manifest_ref,
            file_dialog: FileDialog::new(),
            is_profile_window_open: false,
            download_result_channel: Channel::new(100),
            download_progress_channel: Channel::new(500),
            download_total_channel: Channel::new(100),
            tabs_state: TabsState(tabs),
        }
    }
}

impl TabViewer for MyContext {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Profiles {
                profiles_state,
                menu_state,
            } => ProfilesPage {
                download_result_tx: self.download_result_channel.clone_tx(),
                download_progress_tx: self.download_progress_channel.clone_tx(),
                download_total_tx: self.download_total_channel.clone_tx(),

                state: profiles_state,
                menu_state,

                launcher_manifest: self.launcher_manifest,
                is_profile_window_open: &mut self.is_profile_window_open,
            }
            .ui(ui),
            Tab::Settings => SettingsPage {
                storage: &mut self.storage,
                file_dialog: &mut self.file_dialog,
            }
            .ui(ui),
            Tab::Logs => {
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.add(egui_tracing::Logs::new(self.collector.clone()));
                });
            }
            Tab::DownloadStatus => {
                DownloadProgress {
                    storage: &mut self.storage,
                    download_result_rx: &mut self.download_result_channel.rx,
                    download_progress_rx: &mut self.download_progress_channel.rx,
                    download_total_rx: &mut self.download_total_channel.rx,
                }
                .ui(ui);
            }
        };
    }
}

struct MyTabs {
    context: MyContext,
    dock_state: DockState<Tab>,
}

impl MyTabs {
    pub fn new(collector: EventCollector) -> Self {
        let tabs = vec![
            Tab::Profiles {
                profiles_state: ProfilesState::default(),
                menu_state: AddProfileMenuState::default(),
            },
            Tab::Settings,
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
        let pixels_per_point = self
            .context
            .storage
            .get::<ClientSettings>()
            .unwrap()
            .pixels_per_point;
        ctx.set_pixels_per_point(pixels_per_point);

        let mut added_nodes = Vec::<Tab>::new();

        egui::TopBottomPanel::top("top_panel_id").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                AddTab {
                    dock_state: &self.dock_state,
                    added_tabs: &mut added_nodes,
                    tabs_state: &mut self.context.tabs_state,
                }
                .ui(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        egui::CentralPanel::default()
            .frame(Frame::central_panel(ctx.style().as_ref()).inner_margin(0.0))
            .show(ctx, |ui| self.ui(ui));

        added_nodes
            .drain(..)
            .for_each(|node| self.dock_state.push_to_first_leaf(node));

        let opened_tabs = self
            .dock_state
            .iter_all_tabs()
            .map(|(_, tab)| tab)
            .map(Tab::id)
            .collect::<Vec<_>>();

        for tab_id in &self.context.tabs_state.0 {
            if !opened_tabs.contains(tab_id) {
                self.dock_state
                    .push_to_first_leaf(Tab::from_id(tab_id.to_owned()))
            }
        }
    }
}
