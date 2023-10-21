use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum JavaRunner<'a> {
    Str(&'a str),
    Path(PathBuf),
}

impl<'a> JavaRunner<'a> {
    pub const STR: JavaRunner<'static> = JavaRunner::Str("java");

    pub fn path(p: PathBuf) -> JavaRunner<'a> {
        JavaRunner::Path(p)
    }

    pub fn str(s: &str) -> JavaRunner<'_> {
        JavaRunner::Str(s)
    }
}

impl<'a> Default for JavaRunner<'a> {
    fn default() -> JavaRunner<'a> {
        JavaRunner::Str("java")
    }
}
