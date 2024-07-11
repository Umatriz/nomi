use crate::loaders::{fabric::Fabric, vanilla::Vanilla};

use super::launch::{LaunchInstanceBuilder, LaunchSettings};

pub trait LaunchInstanceBuilderExt {
    fn insert(&self, builder: LaunchInstanceBuilder<LaunchSettings>) -> LaunchInstanceBuilder<LaunchSettings>;
}

const _: Option<Box<dyn LaunchInstanceBuilderExt>> = None;

impl LaunchInstanceBuilderExt for Vanilla {
    fn insert(&self, builder: LaunchInstanceBuilder<LaunchSettings>) -> LaunchInstanceBuilder<LaunchSettings> {
        builder
    }
}

impl LaunchInstanceBuilderExt for Fabric {
    fn insert(&self, builder: LaunchInstanceBuilder<LaunchSettings>) -> LaunchInstanceBuilder<LaunchSettings> {
        builder.profile(self.to_profile())
    }
}
