use crate::loaders::{combined::VanillaCombinedDownloader, vanilla::Vanilla, ToLoaderProfile};

use super::launch::{LaunchInstanceBuilder, LaunchSettings};

pub trait LaunchInstanceBuilderExt {
    fn insert(&self, builder: LaunchInstanceBuilder<LaunchSettings>) -> LaunchInstanceBuilder<LaunchSettings>;
}

const _: Option<Box<dyn LaunchInstanceBuilderExt>> = None;

// Unique case where we do not have a profile.
// TODO: Maybe make a profile for Vanilla and get rid of manifest?
impl LaunchInstanceBuilderExt for Vanilla {
    fn insert(&self, builder: LaunchInstanceBuilder<LaunchSettings>) -> LaunchInstanceBuilder<LaunchSettings> {
        builder
    }
}

// If the generic is `()` that means we are downloading `Vanilla`
impl LaunchInstanceBuilderExt for VanillaCombinedDownloader<()> {
    fn insert(&self, builder: LaunchInstanceBuilder<LaunchSettings>) -> LaunchInstanceBuilder<LaunchSettings> {
        builder
    }
}

impl<L> LaunchInstanceBuilderExt for L
where
    L: ToLoaderProfile,
{
    fn insert(&self, builder: LaunchInstanceBuilder<LaunchSettings>) -> LaunchInstanceBuilder<LaunchSettings> {
        builder.profile(self.to_profile())
    }
}
