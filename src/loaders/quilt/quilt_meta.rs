use serde::{Deserialize, Serialize};

pub type Meta = Vec<QuiltVersion>;

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct QuiltVersion {}
