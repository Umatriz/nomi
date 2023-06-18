use std::collections::HashSet;

use eframe::egui;
use tokio::runtime::Builder;
use egui_dock::{Tree, Style, Node};

use crate::commands::{
  download_version,
  launch,
  get_manifest
};
use crate::configs::Config;
use crate::utils::GetPath;
use crate::downloads::launcher_manifest::{LauncherManifestVersion};
use crate::configs::launcher::{Profile, Launcher};

impl Config for Launcher {}

pub struct Main {
  pub tree: Tree<String>,
  
  pub context: MyContext,
}

// TODO: Add substructs
pub struct MyContext {
  pub username: String,

  pub launcher_config: Launcher,
  pub selected_profile: usize,

  // TODO: remove
  pub profile_name: String,

  pub versions: Vec<LauncherManifestVersion>,
  pub selected_version: usize,

  pub style: Option<Style>,
  pub open_tabs: HashSet<String>,

  pub show_close_buttons: bool,
  pub show_add_buttons: bool,
  pub draggable_tabs: bool,
  pub show_tab_name_on_hover: bool,
}

impl Default for Main {
  fn default() -> Self {
    let runtime = Builder::new_multi_thread()
      .worker_threads(1)
      .enable_all()
      .build()
      .unwrap();

    let tree = Tree::new(vec!["Launcher".to_owned(), "Profiles".to_owned(),]);

    let mut open_tabs = HashSet::new();

    for node in tree.iter() {
      if let Node::Leaf { tabs, .. } = node {
        for tab in tabs {
          open_tabs.insert(tab.clone());
        }
      }
    }

    let conf = Launcher::from_file(None);

    let context = MyContext {
      style: None,
      open_tabs,
      show_close_buttons: false,
      show_add_buttons: false,
      // TODO: Set to false
      draggable_tabs: true,
      show_tab_name_on_hover: false,
      versions: runtime.block_on(get_manifest()).unwrap(),
      selected_version: 0,
      username: conf.username.clone(),
      launcher_config: conf,
      selected_profile: 0,
      profile_name: String::new(),
    };

    // TODO: add Config support
    Self {
      tree,
      context
    }
  }
}

impl MyContext {
  fn launcher(&mut self, ui: &mut egui::Ui) {
    if self.launcher_config.profiles.is_empty() {
      ui.label("Create profile");
    } else {
      ui.horizontal(|ui| {
        ui.label("Your name: ");
        ui.text_edit_singleline(&mut self.username);
      });
      egui::ComboBox::from_label("Take your pick")
        .selected_text(format!("{}", &self.launcher_config.profiles[self.selected_profile].version))
        .show_ui(ui, |ui| { 
          for i in 0..self.launcher_config.profiles.len() {
            let value = ui.selectable_value(&mut &self.launcher_config.profiles[i], &self.launcher_config.profiles[self.selected_profile], &self.launcher_config.profiles[i].version);
            if value.clicked() {
              self.selected_profile = i;
            }
          }
        });
      ui.end_row();
    }
  }

  fn profiles(&mut self, ui: &mut egui::Ui) {
    ui.heading("Create new Profile");

    ui.horizontal(|ui| {
      ui.label("Profile name: ");
      ui.text_edit_singleline(&mut self.profile_name);
    });

    egui::ComboBox::from_label("Select version (SUPPORTS RELEASE VERSIONS ONLY!)")
      .selected_text(format!("{}", &self.versions[self.selected_version].id))
      .show_ui(ui, |ui| { 
        for i in 0..self.versions.len() {
          let value = ui.selectable_value(&mut &self.versions[i], &self.versions[self.selected_version], &self.versions[i].id);
          if value.clicked() {
            self.selected_version = i;
          }
        }
      });

    if ui.add(egui::Button::new("Click me")).clicked() {
      let profile = Profile::new(
        self.versions[self.selected_version].id.clone(),
        "release".to_string(),
        GetPath::game().to_str().unwrap().to_string(),
        &self.launcher_config.profiles,
        self.profile_name.clone(),
      );
      
      // self.launcher_config.profiles.push(profile);
      self.launcher_config.add_profile(profile);
      self.launcher_config.overwrite(GetPath::config())
    }

    ui.end_row();
  }
}



pub struct TabViewer {}

impl egui_dock::TabViewer for MyContext {
  type Tab = String;

  fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
    match tab.as_str() {
      "Launcher" => self.launcher(ui),
      "Profiles" => self.profiles(ui),
      _ => {
        ui.label(tab.as_str());
      }
    }
  }

  fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
    (&*tab).into()
  }

  fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
    false
  }
}