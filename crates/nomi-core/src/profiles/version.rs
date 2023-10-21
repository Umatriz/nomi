use const_typed_builder::Builder;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    instance::launch::{LaunchInstance, LaunchInstanceBuilder, LaunchSettingsBuilder},
    repository::{java_runner::JavaRunner, username::Username},
};

#[derive(Serialize, Deserialize, Debug, Default, Builder)]
pub struct VersionProfile {
    pub id: i32,
    pub is_downloaded: bool,

    pub version: String,
    pub version_type: String,
    pub version_jar_file: PathBuf,

    pub assets: PathBuf,
    pub game_dir: PathBuf,
    pub libraries_dir: PathBuf,
    pub manifest_file: PathBuf,
    pub profile_file: Option<PathBuf>,
    pub natives_dir: PathBuf,
}

impl VersionProfile {
    pub fn into_launch(
        self,
        username: Username,
        java_bin: JavaRunner<'static>,
        access_token: Option<String>,
        uuid: Option<String>,
    ) -> LaunchInstance<'static> {
        let settings = LaunchSettingsBuilder::new()
            .game_dir(self.game_dir)
            .assets(self.assets)
            .libraries_dir(self.libraries_dir)
            .manifest_file(self.manifest_file)
            .natives_dir(self.natives_dir)
            .version(self.version)
            .version_jar_file(self.version_jar_file)
            .version_type(self.version_type)
            .username(username)
            .access_token(access_token)
            .java_bin(java_bin)
            .uuid(uuid)
            .build();
        LaunchInstanceBuilder::new().settings(settings).build()
    }
}
