use std::collections::HashSet;
use std::sync::Arc;

use eframe::egui;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{mpsc, Mutex};
use egui_dock::{Tree, Style, Node};

use crate::bootstrap::{ClientBootstrap, ClientSettings, ClientAuth, ClientVersion};
use crate::commands::{Commands};
use crate::configs::Config;
use crate::utils::{GetPath, get_java_bin};
use crate::downloads::launcher_manifest::{LauncherManifestVersion};
use crate::configs::launcher::{Profile, Launcher};

impl Config for Launcher {}

pub struct Main {
  pub tree: Tree<String>,
  
  pub context: MyContext,
}

// TODO: Add substructs
pub struct MyContext {
  pub runtime: Runtime,

  pub state: bool,

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
      runtime: Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap(),

      state: false,

      style: None,
      open_tabs,
      show_close_buttons: false,
      show_add_buttons: false,
      // TODO: Set to false
      draggable_tabs: true,
      show_tab_name_on_hover: false,
      versions: runtime.block_on(Commands::get_manifest()).unwrap(),
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
        if ui.add(egui::Button::new("Update username")).clicked() {
          self.launcher_config.update_username(self.username.clone());
          self.launcher_config.overwrite(GetPath::config());
        }
      });
      egui::ComboBox::from_label("Take your pick")
        .selected_text(format!("{}", &self.launcher_config.profiles[self.selected_profile].name))
        .show_ui(ui, |ui| { 
          for i in 0..self.launcher_config.profiles.len() {
            let value = ui.selectable_value(&mut &self.launcher_config.profiles[i], &self.launcher_config.profiles[self.selected_profile], &self.launcher_config.profiles[i].name);
            if value.clicked() {
              self.selected_profile = i;
            }
          }
        });

      if self.launcher_config.profiles[self.selected_profile].is_downloaded {
        let java = get_java_bin();
        match java {
          Some(java) => {
            if ui.add(egui::Button::new("Play")).clicked() {
              let prof: &Profile = &self.launcher_config.profiles[self.selected_profile];
              let bootstrap = ClientBootstrap::new(ClientSettings {
                assets: GetPath::game().join("assets"),
                auth: ClientAuth {
                  username: self.username.clone(),
                  access_token: None,
                  uuid: Some(uuid::Uuid::new_v4().to_string()),
                },
                game_dir: GetPath::game(),
                java_bin: java,
                libraries_dir: GetPath::game().join("libraries"),
                manifest_file: GetPath::game()
                  .join("versions")
                  .join(&prof.version)
                  .join(format!("{}.json", &prof.version)),
                natives_dir: GetPath::game()
                  .join("versions")
                  .join(&prof.version)
                  .join("natives"),
                version: ClientVersion {
                  version: prof.version.clone(),
                  version_type: prof.version_type.clone(),
                },
                version_jar_file: GetPath::game()
                  .join("versions")
                  .join(&prof.version)
                  .join(format!("{}.jar", &prof.version)),
              });
        
              bootstrap.launch().unwrap();
            }
          }
          None => {
            ui.label("Java not found");
          },
        }
      } else {
        if ui.add(egui::Button::new("Download")).clicked() {
          let version = self.launcher_config.profiles[self.selected_profile].version.clone();
          
          // let (tx, mut rx) = mpsc::channel::<bool>(1);

          let (tx, rx) = std::sync::mpsc::channel::<bool>();

          self.state = true;

          // self.runtime.spawn(async move {
          //   dbg!("1");
          //   tokio::time::sleep(tokio::time::Duration::from_millis(7000)).await;
          //   dbg!("2");
          //   tx.send(true).await.unwrap();
          // });

          std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            rt.block_on(async {
              dbg!("1");
              tokio::time::sleep(tokio::time::Duration::from_millis(7000)).await;
              dbg!("2");
            });
            tx.send(true).unwrap();
          });

          if self.state {
            ui.add(egui::Spinner::new());
            if let Ok(_) = rx.try_recv() {
              ui.label("text");
            }
          }

          // let resp = res.join().unwrap();
          
          // dbg!(resp);

          // if self.state {
          //   ui.label("text");
          // }
          
          // self.launcher_config.profiles[self.selected_profile].is_downloaded = true; 
          // self.launcher_config.overwrite(GetPath::config());
        }

        // if rx.recv().unwrap() {
        //   ui.spinner();
        // }
      }
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

    if self.profile_name.trim().is_empty() {
      ui.label("the name cannot be empty");
    } else {
      if ui.add(egui::Button::new("Create profile")).clicked() {
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
    }

    ui.end_row();
  }
}

pub struct TabViewer {}

impl egui_dock::TabViewer for MyContext {
  type Tab = String;

  fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
    let runtime = Builder::new_multi_thread()
      .worker_threads(1)
      .enable_all()
      .build()
      .unwrap();

    let _guard = runtime.enter();
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