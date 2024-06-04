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

pub use consts::*;

pub use regex;
pub use uuid::Uuid;
