#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]
pub mod configs;
pub mod downloads;
pub mod instance;
pub mod loaders;
pub mod repository;

pub mod error;
pub mod utils;

pub mod fs;
pub mod game_paths;
pub mod maven_data;
pub mod state;

pub mod consts;

use std::{future::Future, pin::Pin};

pub use consts::*;

pub use regex;
use sha1::Digest;
pub use uuid::Uuid;

type PinnedFutureWithBounds<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub fn calculate_sha1(data: impl AsRef<[u8]>) -> String {
    let value = sha1::Sha1::digest(data);
    base16ct::lower::encode_string(&value)
}
