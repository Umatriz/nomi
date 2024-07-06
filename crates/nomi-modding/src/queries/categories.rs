//! Get a list of categories.

use serde::{Deserialize, Serialize};

pub type Categories = Vec<Category>;

#[derive(Serialize, Deserialize)]
pub struct Category {
    pub icon: String,
    pub name: String,
    pub project_type: String,
    pub header: String,
}
