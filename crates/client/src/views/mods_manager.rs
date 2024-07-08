use std::collections::HashSet;

use eframe::egui::{self, Button, Color32, Layout, RichText, ScrollArea, Vec2};
use egui_infinite_scroll::{InfiniteScroll, LoadingState};
use nomi_modding::{
    capitalize_first_letters_whitespace_splitted,
    modrinth::{
        categories::{Categories, CategoriesData, Header},
        search::{Facets, Hit, InnerPart, Parts, ProjectType, SearchData},
    },
    Query,
};

use crate::{errors_pool::ErrorPoolExt, ui_ext::UiExt};

use super::View;

pub struct ModManager<'a> {
    pub mod_manager_state: &'a mut ModManagerState,
}

#[derive(Default)]
pub struct ModManagerState {
    pub previous_facets: Facets,
    pub previous_project_type: ProjectType,
    pub scroll: InfiniteScroll<Hit, u32>,
    pub categories: Option<Categories>,
    pub current_project_type: ProjectType,
    pub headers: Vec<(Header, ProjectType)>,
    pub selected_categories: HashSet<String>,
}

fn fix_svg(text: &str) -> String {
    let s = &text[5..];
    format!("<svg xmlns=\"http://www.w3.org/2000/svg\" {s}")
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
            current_project_type: ProjectType::Mod,
            previous_project_type: ProjectType::Mod,
            scroll: Self::create_scroll(None),
        }
    }

    fn create_scroll(facets: Option<Facets>) -> InfiniteScroll<Hit, u32> {
        InfiniteScroll::new().end_loader_async(move |cursor| {
            let facets = facets.clone();
            async move {
                let offset = cursor.unwrap_or(0);

                let data = SearchData::builder().offset(offset);
                let data = match facets {
                    Some(f) => data.facets(f).build(),
                    None => data.build(),
                };
                let query = Query::new(data);
                let search = query.query().await.map_err(|e| format!("{:#?}", e))?;

                Ok((search.hits, Some(offset + 10)))
            }
        })
    }

    pub fn update_scroll_with_facets(&mut self, facets: Option<Facets>) {
        self.scroll = Self::create_scroll(facets);
    }
}

impl View for ModManager<'_> {
    fn ui(self, ui: &mut eframe::egui::Ui) {
        egui::TopBottomPanel::top("mod_manager_top_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                for project_type in ProjectType::iter() {
                    ui.selectable_value(
                        &mut self.mod_manager_state.current_project_type,
                        project_type,
                        capitalize_first_letters_whitespace_splitted(project_type.as_str()),
                    );
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
                            self.mod_manager_state.selected_categories = HashSet::new()
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
                                    let set = &mut self.mod_manager_state.selected_categories;
                                    let name = &category.name;

                                    let mut is_open = set.contains(name);

                                    ui.horizontal(|ui| {
                                        ui.add(egui::Image::from_bytes(
                                            format!("bytes://{}.svg", &name),
                                            fix_svg(&category.icon).as_bytes().to_vec(),
                                        ));

                                        ui.checkbox(&mut is_open, name);
                                    });

                                    let facets = || {
                                        let parts = Parts::from_project_type(
                                            self.mod_manager_state.current_project_type,
                                        );

                                        match (*set).is_empty() {
                                            true => Facets::new(parts),
                                            false => Facets::new(
                                                parts.part(InnerPart::from_vec(
                                                    set.iter()
                                                        .map(InnerPart::format_category)
                                                        .collect(),
                                                )),
                                            ),
                                        }
                                    };

                                    if self.mod_manager_state.previous_facets != facets() {
                                        self.mod_manager_state.previous_facets = facets();
                                        self.mod_manager_state.scroll =
                                            ModManagerState::create_scroll(Some(facets()));
                                    } else if self.mod_manager_state.previous_project_type
                                        != self.mod_manager_state.current_project_type
                                    {
                                        self.mod_manager_state.previous_project_type =
                                            self.mod_manager_state.current_project_type;
                                        self.mod_manager_state.scroll =
                                            ModManagerState::create_scroll(Some(facets()));
                                    }

                                    if is_open {
                                        if !set.contains(name) {
                                            set.insert(name.to_owned());
                                        }
                                    } else {
                                        set.remove(name);
                                    }
                                }
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
                                    let _ = ui.button("Download");
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
