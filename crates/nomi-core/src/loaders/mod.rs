use crate::instance::loader::LoaderProfile;

pub mod combined;
pub mod fabric;
pub mod forge;
pub mod vanilla;

pub trait ToLoaderProfile {
    fn to_profile(&self) -> LoaderProfile;
}
