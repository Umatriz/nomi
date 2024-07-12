use std::{ffi::OsStr, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::utils::path_to_string;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
#[must_use]
pub enum JavaRunner {
    Command(String),
    Path(PathBuf),
}

impl JavaRunner {
    #[must_use]
    pub fn get(&self) -> &dyn AsRef<OsStr> {
        match self {
            JavaRunner::Command(s) => s,
            JavaRunner::Path(p) => p,
        }
    }

    #[must_use]
    pub fn get_string(&self) -> String {
        match self {
            JavaRunner::Command(s) => s.to_string(),
            JavaRunner::Path(p) => path_to_string(p),
        }
    }

    pub fn path(p: PathBuf) -> JavaRunner {
        JavaRunner::Path(p)
    }

    pub fn command(s: &str) -> JavaRunner {
        JavaRunner::Command(s.to_string())
    }
}

impl Default for JavaRunner {
    fn default() -> JavaRunner {
        JavaRunner::Command("java".to_string())
    }
}
