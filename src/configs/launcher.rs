use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::configs::{Config, ConfigFile};

// is this a good way?
struct ConfigDir;

impl ConfigDir {
  pub fn new() -> PathBuf {
    // TODO: Remove this .join()
    std::env::current_dir().unwrap().join("config.yaml")
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct LauncherConfig {
  pub username: String,
  pub profiles: Vec<ProfileConfig>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct ProfileConfig {
  id: i32,
  pub version: String,
  pub version_type: String,
  pub path: String,
}

impl ProfileConfig {
  pub fn new(
    version: String,
    version_type: String,
    path: String,
    profiles: &Vec<ProfileConfig>
  ) -> Self {
    Self {
      id: Self::create_id(&profiles),
      version,
      version_type,
      path,
    }
  }

  pub fn create_id(profiles: &Vec<ProfileConfig>) -> i32 {
    let mut max_id: Vec<i32> = vec![];
    for prof in profiles.iter() {
      max_id.push(prof.id)
    }

    match max_id.iter().max() {
      Some(mx) => dbg!(mx + 1),
      None => {
        println!("Vec is empty");
        0
      }
    }
  }
}

impl Config for LauncherConfig {}

impl LauncherConfig {
  pub fn from_file(username: Option<String>) -> Self {
    let conf: ConfigFile = ConfigFile::new(ConfigDir::new());
    match conf.0 {
      true => {
        let f = std::fs::File::open(conf.1).expect("Could not open file");
        let mut read: Self = serde_yaml::from_reader(f).expect("Could not read values");
        match username {
          Some(u) => {
            read.username = u;
            read
          },
          None => read,
        }
      },
      false => {
        Self {
          username: match username {
            Some(u) => u,
            None => "nomi".to_string(),
        },
          profiles: vec![],
        }
      },
    }
  }

  pub fn add_profile(&mut self, profile: ProfileConfig) {
    self.profiles.push(profile);
  }

  pub fn get_profile(&self, id: i32) -> Option<&ProfileConfig> {
    for prof in self.profiles.iter() {
      if prof.id == id {
        return Some(prof);
      }
    }
    return None;
  }

  pub fn remove_profile(&mut self, id: usize) {
    self.profiles.remove(id);
  }

  pub fn update_username(&mut self, username: String) {
    self.username = username
  }
}
