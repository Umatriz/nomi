use serde::{Deserialize, Serialize};

use super::version::VersionProfile;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct User {
    pub username: String,
    pub profiles: Vec<VersionProfile>,
}

impl User {
    pub fn add_profile(
        &mut self,
        version: String,
        version_type: String,
        path: String,
        name: String,
    ) {
        self.profiles.push(VersionProfile {
            id: self.create_id(),
            version,
            version_type,
            path,
            name,
            is_downloaded: false,
        })
    }

    fn create_id(&self) -> i32 {
        let mut max_id: Vec<i32> = vec![];
        for prof in self.profiles.iter() {
            max_id.push(prof.id)
        }

        match max_id.iter().max() {
            Some(mx) => mx + 1,
            None => 0,
        }
    }
}
