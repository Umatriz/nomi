use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{mpsc::Sender, Arc},
};

use eframe::egui::{
    self, Button, Color32, ComboBox, Id, Image, Layout, RichText, ScrollArea, Vec2,
};
use egui_infinite_scroll::{InfiniteScroll, LoadingState};
use egui_task_manager::{Caller, Progress, Task, TaskManager, TaskProgressShared};
use itertools::Itertools;
use nomi_core::{
    calculate_sha1, downloads::{progress::MappedSender, traits::Downloader, DownloadSet, FileDownloader}, DOT_NOMI_DATA_PACKS_DIR, MINECRAFT_DIR
};
use nomi_modding::{
    capitalize_first_letters_whitespace_splitted,
    modrinth::{
        categories::{Categories, CategoriesData, Header},
        project::{Project, ProjectData, ProjectId},
        search::{Facets, Hit, InnerPart, Parts, ProjectType, SearchData},
        version::{Dependency, File, ProjectVersionsData, Version, VersionId},
    },
    Query,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    collections::{
        DependenciesCollection, ModsDownloadingCollection, ProjectCollection,
        ProjectVersionsCollection,
    },
    errors_pool::ErrorPoolExt,
    progress::UnitProgress,
    ui_ext::UiExt,
    DOT_NOMI_MODS_STASH_DIR, MINECRAFT_MODS_DIRECTORY,
};

use super::{ModdedProfile, ProfilesConfig, View};

pub use crate::mods::*;

pub struct ModManager<'a> {
    pub task_manager: &'a mut TaskManager,
    pub profiles_config: &'a mut ProfilesConfig,
    pub profile: Arc<ModdedProfile>,
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

    pub is_download_window_open: bool,
    pub download_window_state: DownloadWindowState,
    pub current_project: Option<Project>,
    pub current_versions: Vec<Arc<Version>>,
    pub selected_version: Option<Arc<Version>>,
    pub current_dependencies: Vec<SimpleDependency>,
    pub selected_dependencies: HashMap<String, MaybeAddedDependency>,
}

pub struct MaybeAddedDependency {
    version: Option<Arc<Version>>,
    is_added: bool,
}

#[derive(Default)]
pub struct DownloadWindowState {
    project_id: ProjectId,
    game_version: String,
    loader: String,
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
            download_window_state: DownloadWindowState::default(),
            is_download_window_open: false,
            current_project: None,
            current_versions: Vec::new(),
            selected_version: None,
            current_dependencies: Vec::new(),
            selected_dependencies: HashMap::new(),
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
                for project_type in ProjectType::iter().filter(|t| !matches!(t, ProjectType::Plugin)) {
                    let response = ui.selectable_value(
                        &mut self.mod_manager_state.current_project_type,
                        project_type,
                        capitalize_first_letters_whitespace_splitted(project_type.as_str()),
                    );

                    if response.clicked() {
                        self.mod_manager_state.clear_filter()
                    }
                }

                match self.mod_manager_state.current_project_type {
                    ProjectType::Shader => {
                        ui.warn_label_with_icon_before("You need to have a way to support shader.").on_hover_text("By default shaders are downloaded into minecraft/shaderpacks.");
                    },
                    ProjectType::DataPack => {
                        ui.warn_label_with_icon_before("You need to manually add datapacks to your world. Open -> Data Packs.").on_hover_text("You can open a datapacks folder by clicking Open -> Data Packs in the top menu.");
                    },
                    _ => ()
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

                                    ui.horizontal(|ui| {
                                        if self
                                            .profile
                                            .mods
                                            .mods
                                            .iter()
                                            .any(|m| m.project_id == item.project_id)
                                        {
                                            ui.colored_label(Color32::GREEN, "✅").on_hover_text("This mod is already downloaded. Downloading it again will replace files.");
                                        }

                                        if ui.button("Download").clicked() {
                                            self.mod_manager_state.is_download_window_open = true;

                                            let game_version =
                                                self.profile.profile.version().to_owned();

                                            let loader =
                                                self.profile.profile.loader_name().to_lowercase();

                                            self.mod_manager_state.download_window_state =
                                                DownloadWindowState {
                                                    project_id: item.project_id.clone(),
                                                    game_version: game_version.clone(),
                                                    loader: loader.clone(),
                                                };

                                            let id = item.project_id.clone();
                                            let get_project = Task::new(
                                                "Get project",
                                                Caller::standard({
                                                    async move {
                                                        let query = Query::new(ProjectData::new(
                                                            id.clone(),
                                                        ));
                                                        query.query().await.report_error()
                                                    }
                                                }),
                                            );

                                            self.task_manager
                                                .push_task::<ProjectCollection>(get_project);

                                            self.mod_manager_state.current_versions = Vec::new();

                                            let id = item.project_id.clone();
                                            let get_versions = Task::new(
                                                "Get project",
                                                Caller::standard(async move {
                                                    let query = Query::new(
                                                        ProjectVersionsData::builder()
                                                            .id_or_slug(id)
                                                            .game_versions(vec![game_version])
                                                            .loaders(vec![loader])
                                                            .build(),
                                                    );
                                                    query.query().await.report_error()
                                                }),
                                            );

                                            self.task_manager
                                                .push_task::<ProjectVersionsCollection>(
                                                    get_versions,
                                                );
                                        }
                                    });
                                });
                            });
                        });
                    });
            });

            if let LoadingState::Error(err) = self.mod_manager_state.scroll.bottom_loading_state() {
                ui.error_label(err);
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

        egui::Window::new("Mod")
            .open(&mut self.mod_manager_state.is_download_window_open)
            .show(ui.ctx(), |ui| {
                if let Some(project) = &self.mod_manager_state.current_project {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                Image::new(&project.icon_url).fit_to_exact_size(Vec2::splat(50.0)),
                            );
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&project.title).heading());
                                ui.label(&project.description)
                            });
                        });

                        ui.separator();

                        ComboBox::from_label("Select mod version")
                            .selected_text(
                                self.mod_manager_state
                                    .selected_version
                                    .as_ref()
                                    .map_or("Select version from the list", |v| &v.version_number),
                            )
                            .show_ui(ui, |ui| {
                                for version in &self.mod_manager_state.current_versions {
                                    let response = ui.selectable_value(
                                        &mut self.mod_manager_state.selected_version,
                                        Some(version.clone()),
                                        version.version_number.clone(),
                                    );

                                    if response.clicked() {
                                        let game_version =
                                            self.profile.profile.version().to_owned();
                                        let loader =
                                            self.profile.profile.loader_name().to_lowercase();
                                        let version = version.clone();

                                        let get_dependencies = Task::new(
                                            "Get dependencies",
                                            Caller::standard(async move {
                                                let mut deps = Vec::new();
                                                proceed_deps(
                                                    &mut deps,
                                                    version.clone(),
                                                    game_version,
                                                    loader,
                                                )
                                                .await
                                                .report_error()
                                                .map(|_| deps)
                                            }),
                                        );

                                        self.task_manager
                                            .push_task::<DependenciesCollection>(get_dependencies);
                                    }
                                }
                            });

                        if !self.mod_manager_state.current_dependencies.is_empty() {
                            ui.separator();

                            ui.label(RichText::new("Dependencies").strong());
                        }

                        for dep in &self.mod_manager_state.current_dependencies {
                            let is_installed = self.profile.mods.mods.iter().any(|m| dep.versions.first().is_some_and(|d| m.project_id == d.project_id));

                            let is_added = if is_installed {
                                false
                            } else {
                                dep.is_required
                            };

                            if !self
                                .mod_manager_state
                                .selected_dependencies
                                .contains_key(&dep.name)
                            {
                                self.mod_manager_state.selected_dependencies.insert(
                                    dep.name.clone(),
                                    MaybeAddedDependency {
                                        version: None,
                                        is_added,
                                    },
                                );
                            }
                            let val = self
                                .mod_manager_state
                                .selected_dependencies
                                .get_mut(&dep.name)
                                .unwrap();

                            ui.horizontal(|ui| {
                                if is_installed {
                                    ui.colored_label(Color32::GREEN, "✅").on_hover_text("This dependency is already downloaded. If you want to download it again do it manually.");
                                }

                                ui.add_enabled_ui(!is_installed, |ui| {
                                    ui.add_enabled_ui(!dep.is_required, |ui| {
                                        ui.checkbox(&mut val.is_added, "")
                                            .on_hover_text("Include this dependency");
                                    });
    
                                    ui.add_enabled_ui(val.is_added, |ui| {
                                        ui.label(&dep.name);
    
                                        ComboBox::from_id_source(Id::new(&dep.name))
                                            .selected_text(
                                                val.version
                                                    .clone()
                                                    .map_or("No version selected".to_owned(), |v| {
                                                        v.version_number.clone()
                                                    })
                                                    .to_string(),
                                            )
                                            .show_ui(ui, |ui| {
                                                for version in &dep.versions {
                                                    ui.horizontal(|ui| {
                                                        if version.featured {
                                                            ui.colored_label(Color32::GREEN, "✅")
                                                                .on_hover_text(
                                                                "This version is featured by the author",
                                                            );
                                                        }
                                                        ui.selectable_value(
                                                            &mut val.version,
                                                            Some(version.clone()),
                                                            version.version_number.clone(),
                                                        );
                                                    });
                                                }
                                            });
                                    });
                                });
                            });
                        }

                        let is_version_selected = self.mod_manager_state.selected_version.is_some();
                        if !is_version_selected {
                            ui.error_label("You must select the version");
                        }

                        let is_dependencies_selected = self.mod_manager_state.selected_dependencies.values().filter(|d| d.is_added).all(|d| d.version.is_some());
                        if !is_dependencies_selected {
                            ui.error_label("Select version for all included dependencies");
                        }

                        if ui.add_enabled(is_version_selected && is_dependencies_selected, Button::new("Download")).clicked() {
                            let directory = match self.mod_manager_state.current_project_type {
                                ProjectType::Mod | ProjectType::Modpack => PathBuf::from(DOT_NOMI_MODS_STASH_DIR).join(format!("{}", self.profile.profile.id)),
                                ProjectType::ResourcePack => PathBuf::from(MINECRAFT_DIR).join("resourcepacks"),
                                ProjectType::Shader => PathBuf::from(MINECRAFT_DIR).join("shaderpacks"),
                                ProjectType::DataPack => PathBuf::from(DOT_NOMI_DATA_PACKS_DIR),
                                _ => unreachable!("You cannot download plugins")
                            };

                            let mut versions = vec![self.mod_manager_state.selected_version.clone().unwrap()];
                            versions.extend(self.mod_manager_state.selected_dependencies.values().filter(|d| d.is_added).filter_map(|d| d.version.clone()));

                            let profile_id = self.profile.profile.id;

                            let _ = self.profiles_config.update_config().report_error();
                            let download_mod = Task::new("Download mods", Caller::progressing(move |progress| async move {
                                let mods = download_mods(progress, directory, versions).await;
                                let mut cfg = ProfilesConfig::read_async().await;

                                // PANICS: Will never panic since this page cannot be accessed without profile.
                                let profile = cfg.profiles.iter_mut().find(|p| p.profile.id == profile_id).unwrap();

                                if let Some(profile) = Arc::get_mut(profile) {
                                    profile.mods.mods.extend(mods);
                                    debug!("Added mods to profile {profile_id} successfully");
                                }

                                cfg.update_config_async().await.report_error()
                            }));

                           self.task_manager.push_task::<ModsDownloadingCollection>(download_mod);
                        };
                    });
                }
            });
    }
}

