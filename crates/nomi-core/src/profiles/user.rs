use serde::{Deserialize, Serialize};

use super::version::VersionProfile;

/// `Settings` its a global settings of the launcher
/// currently have only 2 fields
/// `usernames` - vector of saved usernames
/// `profiles` - vector of existing profiles
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Settings {
    pub usernames: Vec<String>,
    pub profiles: Vec<VersionProfile>,
}

impl Settings {
    /// Expects a mutable reference to `Self`
    /// creates new `VersionProfile` and pushes it to the `profiles` field
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

    /// create id for profile
    /// depends on the last id in the vector
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

#[cfg(test)]
mod tests {
    use crate::profiles::{read_config, write_config};

    use super::*;

    #[tokio::test]
    async fn write_test() {
        let mock = Settings {
            usernames: vec!["test".to_owned(), "test2".to_owned()],
            profiles: vec![
                VersionProfile {
                    id: 0,
                    version: "1.18.2".to_owned(),
                    version_type: "release".to_owned(),
                    path: "./minecraft".to_owned(),
                    name: "Name1".to_owned(),
                    is_downloaded: false,
                },
                VersionProfile {
                    id: 1,
                    version: "1.18.2".to_owned(),
                    version_type: "release".to_owned(),
                    path: "./minecraft".to_owned(),
                    name: "Name2".to_owned(),
                    is_downloaded: false,
                },
            ],
        };

        write_config(&mock, "./configs/Settings.toml")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn read_test() {
        let data: Settings = read_config("./configs/Settings.toml").await.unwrap();

        dbg!(data);
    }
}
