use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  pub username: String,
  pub profiles: Vec<Profile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
  id: i32,
  pub version: String,
  pub version_type: String,
  pub path: String,
}

impl Profile {
  pub fn new(
    version: String,
    version_type: String,
    path: String,
    profiles: Vec<Profile>
  ) -> Self {
    Self {
      id: Self::create_id(profiles),
      version,
      version_type,
      path,
    }
  }

  pub fn create_id(profiles: Vec<Profile>) -> i32 {
    let mut max_id: Vec<i32> = vec![];
    for prof in profiles.iter() {
      max_id.push(prof.id)
    }

    match max_id.iter().max() {
      Some(mx) => mx + 1,
      None => {
        println!("Vec is empty");
        0
      }
    }
  }
}

impl Config {
  pub fn new(username: String) -> Self {
    Self {
      username,
      profiles: vec![],
    }
  }

  pub fn does_exist(&self) -> (bool, Option<PathBuf>) {
    let config = std::env::current_dir().unwrap().join("config.yaml");
    if config.exists() {
      return (true, Some(config))
    }
    return (false, None);
  }

  pub fn write_config(&self) -> Result<(), serde_yaml::Error> {
    if self.does_exist().0 {
      print!("Config already exist");
      Ok(())
    } else {
      let file = std::fs::File::create(self.does_exist().1.unwrap()).unwrap();
      let _ = serde_yaml::to_writer(&file, &self);
      println!("created config");
      Ok(())
    }
  }

  pub fn read_config(&self) {
    todo!()
  }

  pub fn add_profile(&mut self, profile: Profile) {
    self.profiles.push(profile)
  }

  pub fn get_profile(&self, id: i32) -> Option<&Profile> {
    for prof in self.profiles.iter() {
      if prof.id == id {
        return Some(prof);
      }
    }
    return None;
  }
}