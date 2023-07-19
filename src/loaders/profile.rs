use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub struct LoaderProfileArguments {
    pub game: Option<Vec<String>>,
    pub jvm: Option<Vec<String>>,
}

///It is used in `bootstrap` to return a profile
pub trait LoaderProfile {
    fn read_from_file(&self, path: PathBuf) -> anyhow::Result<Self>
    where
        Self: Sized + for<'de> Deserialize<'de> + Serialize,
    {
        let f = std::fs::File::open(path).expect("Could not open file");
        let read: Self = serde_json::from_reader(f).expect("Could not read values");

        Ok(read)
    }

    fn get_args(&self) -> LoaderProfileArguments;

    fn get_main_class(&self) -> String;
}
