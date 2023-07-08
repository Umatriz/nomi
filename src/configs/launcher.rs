use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::configs::ConfigFile;

use crate::utils::GetPath;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Launcher {
    pub username: String,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Profile {
    id: i32,
    pub version: String,
    pub version_type: String,
    pub path: String,
    pub name: String,
    pub is_downloaded: bool,
}

impl Profile {
    pub fn empty() -> Self {
        Self {
            id: 0,
            version: String::new(),
            version_type: String::new(),
            path: String::new(),
            name: String::new(),
            is_downloaded: false,
        }
    }
    pub fn new(
        version: String,
        version_type: String,
        path: String,
        profiles: &[Profile],
        name: String,
    ) -> Self {
        Self {
            id: Self::create_id(profiles),
            version,
            version_type,
            path,
            name,
            is_downloaded: false,
        }
    }

    pub fn create_id(profiles: &[Profile]) -> i32 {
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

impl Launcher {
    pub fn from_file(username: Option<String>) -> Result<Self> {
        let conf: ConfigFile = ConfigFile::new(GetPath::config()?);
        match conf.0 {
            true => {
                let f = std::fs::File::open(conf.1).expect("Could not open file");
                let mut read: Self = serde_json::from_reader(f).expect("Could not read values");
                match username {
                    Some(u) => {
                        read.username = u;
                        Ok(read)
                    }
                    None => Ok(read),
                }
            }
            false => Ok(Self {
                username: match username {
                    Some(u) => u,
                    None => "nomi".to_string(),
                },
                profiles: vec![],
            }),
        }
    }

    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.push(profile);
    }

    pub fn get_profile(&self, id: i32) -> Option<&Profile> {
        self.profiles.iter().find(|&prof| prof.id == id)
    }

    pub fn remove_profile(&mut self, id: usize) {
        self.profiles.remove(id);
    }

    pub fn update_username(&mut self, username: String) {
        self.username = username
    }
}
