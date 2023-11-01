use std::{ffi::OsStr, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::utils::path_to_string;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum JavaRunner {
    String(String),
    Path(PathBuf),
}

impl JavaRunner {
    pub fn get(&self) -> &dyn AsRef<OsStr> {
        match self {
            JavaRunner::String(s) => s,
            JavaRunner::Path(p) => p,
        }
    }

    pub fn get_string(&self) -> String {
        match self {
            JavaRunner::String(s) => s.to_string(),
            JavaRunner::Path(p) => path_to_string(p),
        }
    }

    pub fn path(p: PathBuf) -> JavaRunner {
        JavaRunner::Path(p)
    }

    pub fn str(s: &str) -> JavaRunner {
        JavaRunner::String(s.to_string())
    }
}

impl Default for JavaRunner {
    fn default() -> JavaRunner {
        JavaRunner::String("java".to_string())
    }
}
