use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::loaders::{
    fabric::fabric_meta::FabricProfile, profile::LoaderProfile, quilt::quilt_meta::QuiltProfile,
};

pub enum Loader {
    Quilt,
    Forge,
    Fabric,
    Vanilla,
}

pub enum Profile {
    Quilt(Box<dyn LoaderProfile>),
    Fabric(Box<dyn LoaderProfile>),
    None,
}

impl Loader {
    pub fn load_profile(&self, path: PathBuf) -> anyhow::Result<Profile> {
        match self {
            Loader::Quilt => Ok(Profile::Quilt(Box::new(
                QuiltProfile::default().read_from_file(path)?,
            ))),
            Loader::Forge => Ok(Profile::None),
            Loader::Fabric => Ok(Profile::Quilt(Box::new(
                FabricProfile::default().read_from_file(path)?,
            ))),
            Loader::Vanilla => Ok(Profile::None),
        }
    }
}

impl Profile {
    pub fn unwrap(&self) -> Option<&dyn LoaderProfile> {
        match self {
            Profile::Quilt(quilt) => Some(quilt.as_ref()),
            Profile::Fabric(fabric) => Some(fabric.as_ref()),
            Profile::None => None,
        }
    }
}
