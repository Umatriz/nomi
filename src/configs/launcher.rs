use serde::{Serialize, Deserialize};

use crate::configs::Config;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
struct LauncherConfig {
  username: String,
  profiles: ProfileConfig
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
  
}
