use super::fabric_profile::Arguments;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimpleArgs {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

impl From<&Arguments> for SimpleArgs {
    fn from(value: &Arguments) -> Self {
        let mut args = SimpleArgs {
            game: vec![],
            jvm: vec![],
        };
        value.game.iter().for_each(|a| args.game.push(a.to_owned()));
        value.jvm.iter().for_each(|a| args.jvm.push(a.to_owned()));
        args
    }
}
