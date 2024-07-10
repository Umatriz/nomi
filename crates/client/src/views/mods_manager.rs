use std::{collections::HashSet, path::PathBuf, sync::mpsc::Sender};

use eframe::egui::{
    self, popup_below_widget, scroll_area::ScrollBarVisibility, Button, Color32, Id, Layout,
    PopupCloseBehavior, RichText, ScrollArea, SizeHint, TextureOptions, Vec2,
};
use egui_extras::{Column, Table, TableBuilder};
use egui_infinite_scroll::{InfiniteScroll, LoadingState};
use egui_task_manager::{Progress, TaskProgressShared};
use nomi_core::{
    configs::profile::Loader,
    downloads::{progress::MappedSender, traits::Downloader, DownloadSet, FileDownloader},
};
use nomi_modding::{
    capitalize_first_letters_whitespace_splitted,
    modrinth::{
        categories::{Categories, CategoriesData, Header},
        dependencies::DependenciesData,
        project::{Project, ProjectData, ProjectId},
        search::{Facets, Hit, InnerPart, Parts, ProjectType, SearchData},
        version::{Dependency, ProjectVersionsData, SingleVersionData, Version, VersionId},
    },
    Query,
};

use crate::{errors_pool::ErrorPoolExt, ui_ext::UiExt};

use super::{TabsState, View};

pub struct ModManager<'a> {
    pub current_game_version: String,
    pub current_loader: Loader,
    pub mod_manager_state: &'a mut ModManagerState,
}

#[derive(Default)]
pub struct ModManagerState {
    pub previous_facets: Facets,
    pub previous_project_type: ProjectType,
    pub previous_search: String,
    pub current_search: String,
    pub scroll: InfiniteScroll<Hit, u32>,
    pub categories: Option<Categories>,
    pub current_project_type: ProjectType,
    pub headers: Vec<(Header, ProjectType)>,
    pub selected_categories: HashSet<String>,

    pub download_window_state: DownloadWindow,
}

#[derive(Default)]
pub enum DownloadWindow {
    #[default]
    Closed,
    Open {
        project_id: ProjectId,
        game_version: String,
        loader: String,
    },
}

fn fix_svg(text: &str, color: Color32) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let text = text
        .replace("currentColor", &color.to_hex())
        .replace(']', "");
    let s = &text[5..];
    Some(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" {s}"))
}

impl ModManagerState {
    pub fn new() -> Self {
        let categories = pollster::block_on(async {
            let query = Query::new(CategoriesData);
            query.query().await.report_error()
        });

        let headers = categories
            .as_ref()
            .map(|c| c.get_unique_headers_with_project_type())
            .unwrap_or_default();

        Self {
            categories,
            headers,
            previous_facets: Facets::from_project_type(ProjectType::Mod),
            selected_categories: HashSet::new(),
            previous_search: String::new(),
            current_search: String::new(),
            current_project_type: ProjectType::Mod,
            previous_project_type: ProjectType::Mod,
            scroll: Self::create_scroll(None, None),
            download_window_state: DownloadWindow::Closed,
        }
    }

    fn create_scroll(facets: Option<Facets>, query: Option<String>) -> InfiniteScroll<Hit, u32> {
        InfiniteScroll::new().end_loader_async(move |cursor| {
            let facets = facets.clone();
            let query = query.clone();
            async move {
                let offset = cursor.unwrap_or(0);

                let data = SearchData::builder().offset(offset);
                let mut data = match facets {
                    Some(f) => data.facets(f).build(),
                    None => data.build(),
                };

                data.set_query(query);

                let query = Query::new(data);
                let search = query.query().await.map_err(|e| format!("{:#?}", e))?;

                Ok((search.hits, Some(offset + 10)))
            }
        })
    }

    pub fn update_scroll_with_facets(&mut self, facets: Option<Facets>, query: Option<String>) {
        self.scroll = Self::create_scroll(facets, query);
    }

    pub fn clear_filter(&mut self) {
        self.selected_categories = HashSet::new();
        self.current_search = String::new();
    }
}

impl View for ModManager<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        egui::TopBottomPanel::top("mod_manager_top_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                for project_type in ProjectType::iter() {
                    let response = ui.selectable_value(
                        &mut self.mod_manager_state.current_project_type,
                        project_type,
                        capitalize_first_letters_whitespace_splitted(project_type.as_str()),
                    );

                    if response.clicked() {
                        self.mod_manager_state.clear_filter()
                    }
                }
            });
        });

        egui::SidePanel::left("mod_manager_left_panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        if ui
                            .add_enabled(
                                !self.mod_manager_state.selected_categories.is_empty(),
                                Button::new("Clear filters"),
                            )
                            .clicked()
                        {
                            self.mod_manager_state.clear_filter()
                        }

                        if let Some(categories) = &self.mod_manager_state.categories {
                            let current = self.mod_manager_state.current_project_type;
                            let project_type = if current == ProjectType::Plugin
                                || current == ProjectType::DataPack
                            {
                                ProjectType::Mod
                            } else {
                                current
                            };

                            let set = &mut self.mod_manager_state.selected_categories;

                            for (header, project_type) in self
                                .mod_manager_state
                                .headers
                                .iter()
                                .filter(|(_, t)| *t == project_type)
                            {
                                ui.label(RichText::new(&**header).strong());
                                ui.separator();

                                for category in categories.filter_by_header_and_project_type(
                                    header.clone(),
                                    *project_type,
                                ) {
                                    let name = &category.name;

                                    let mut is_open = set.contains(name);

                                    ui.horizontal(|ui| {
                                        if let Some(svg) = fix_svg(&category.icon, Color32::WHITE) {
                                            ui.add(
                                                egui::Image::from_bytes(
                                                    format!(
                                                        "bytes://{}/{}.svg",
                                                        &category.header, &name
                                                    ),
                                                    svg.as_bytes().to_vec(),
                                                )
                                                .fit_to_original_size(
                                                    1. / ui.ctx().pixels_per_point(),
                                                )
                                                .max_size(Vec2::splat(18.0)),
                                            );
                                        }

                                        ui.checkbox(&mut is_open, name);
                                    });

                                    if is_open {
                                        if !set.contains(name) {
                                            set.insert(name.to_owned());
                                        }
                                    } else {
                                        set.remove(name);
                                    }
                                }
                            }
                            let facets = || {
                                let mut parts = Parts::from_project_type(
                                    self.mod_manager_state.current_project_type,
                                );

                                if !(*set).is_empty() {
                                    parts.add_part(InnerPart::from_vec(
                                        set.iter().map(InnerPart::format_category).collect(),
                                    ));
                                }

                                Facets::new(parts)
                            };

                            let query = || {
                                (!self.mod_manager_state.current_search.is_empty())
                                    .then_some(self.mod_manager_state.current_search.clone())
                            };

                            if self.mod_manager_state.previous_facets != facets() {
                                self.mod_manager_state.previous_facets = facets();
                                self.mod_manager_state.scroll =
                                    ModManagerState::create_scroll(Some(facets()), query());
                            } else if self.mod_manager_state.previous_project_type
                                != self.mod_manager_state.current_project_type
                            {
                                self.mod_manager_state.previous_project_type =
                                    self.mod_manager_state.current_project_type;
                                self.mod_manager_state.scroll =
                                    ModManagerState::create_scroll(Some(facets()), query());
                            } else if self.mod_manager_state.previous_search
                                != self.mod_manager_state.current_search
                            {
                                self.mod_manager_state
                                    .previous_search
                                    .clone_from(&self.mod_manager_state.current_search);
                                self.mod_manager_state.scroll =
                                    ModManagerState::create_scroll(Some(facets()), query())
                            }
                        } else {
                            ui.error_label("Unable to get categories");
                        }
                    });
                });
            });

        ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            if ui.button("Reset").clicked() {
                self.mod_manager_state.scroll.reset()
            }

            ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.set_width(ui.available_width() / 2.0);

                ui.text_edit_singleline(&mut self.mod_manager_state.current_search);

                self.mod_manager_state
                    .scroll
                    .ui(ui, 10, |ui, _index, item| {
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Image::new(&item.icon_url)
                                        .fit_to_exact_size(Vec2::splat(50.0)),
                                );
                                ui.vertical(|ui| {
                                    ui.label(&item.title);
                                    ui.label(&item.description);

                                    if ui.button("Download").clicked() {
                                        // self.mod_manager_state.download_window_state =
                                        //     DownloadWindow::Open {
                                        //         project_id: item.project_id.clone(),
                                        //         game_version: (),
                                        //         loader: (),
                                        //     };
                                    }
                                });
                            });
                        });
                    });
            });

            if let LoadingState::Error(err) = self.mod_manager_state.scroll.bottom_loading_state() {
                ui.label(RichText::new(err).color(ui.visuals().error_fg_color));
            }

            if self
                .mod_manager_state
                .scroll
                .bottom_loading_state()
                .loading()
            {
                ui.spinner();
            }
        });
    }
}

struct ModsConfig {
    mods: Vec<Mod>,
}

struct Mod {
    name: String,
    version_id: VersionId,
    is_installed: bool,
    files: Vec<ModFile>,
    dependencies: Vec<Dependency>,
}

struct ModFile {
    sha1: String,
    url: String,
    filename: String,
}

async fn download_mods(
    progress: TaskProgressShared,
    dir: PathBuf,
    versions: Vec<Version>,
) -> Vec<Mod> {
    let _ = progress.set_total(versions.iter().map(|v| v.files.len() as u32).sum());
    let mut mods = Vec::new();
    for version in versions {
        let mod_value = download_mod(progress.sender(), dir.clone(), version).await;
        mods.push(mod_value);
    }
    mods
}

async fn download_mod(sender: Sender<Box<dyn Progress>>, dir: PathBuf, version: Version) -> Mod {
    let mut set = DownloadSet::new();

    // We do not download any dependencies. Just the mod.
    for file in &version.files {
        let downloader = FileDownloader::new(file.url.clone(), dir.join(&file.filename))
            .with_sha1(file.hashes.sha1.clone());
        set.add(Box::new(downloader));
    }

    let files = version
        .files
        .iter()
        .map(|f| ModFile {
            sha1: f.hashes.sha1.clone(),
            url: f.url.clone(),
            filename: f.filename.clone(),
        })
        .collect::<Vec<_>>();

    let sender = MappedSender::new_progress_mapper(Box::new(sender));

    Box::new(set).download(&sender).await;

    Mod {
        name: version.name,
        version_id: version.id,
        is_installed: true,
        files,
        dependencies: version.dependencies,
    }
}
