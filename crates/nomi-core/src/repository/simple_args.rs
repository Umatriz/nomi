pub struct SimpleArgs {
    pub game: Vec<String>,
    pub jvm: Vec<String>,
}

impl From<super::fabric_profile::Arguments> for SimpleArgs {
    fn from(value: super::fabric_profile::Arguments) -> Self {
        let mut args = SimpleArgs {
            game: vec![],
            jvm: vec![],
        };
        value.game.iter().for_each(|a| args.game.push(a.to_owned()));
        value.jvm.iter().for_each(|a| args.jvm.push(a.to_owned()));
        args
    }
}
