use super::launch::{LaunchInstanceBuilder, LaunchSettings};

pub trait LaunchInstanceBuilderExt {
    fn insert(&self, builder: LaunchInstanceBuilder<LaunchSettings>) -> LaunchInstanceBuilder<LaunchSettings>;
}

const _: Option<Box<dyn LaunchInstanceBuilderExt>> = None;
