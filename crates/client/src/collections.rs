use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use egui_task_manager::*;
use nomi_core::{configs::profile::VersionProfile, repository::fabric_meta::FabricVersions};
use nomi_modding::modrinth::{
    project::Project,
    version::{Version, VersionId},
};

use crate::{
    errors_pool::ErrorPoolExt,
    views::{ModdedProfile, ModsConfig, ProfilesConfig, SimpleDependency},
};

pub struct FabricDataCollection;

impl<'c> TasksCollection<'c> for FabricDataCollection {
    type Context = &'c mut FabricVersions;

    type Target = Option<FabricVersions>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Fabric data collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|value| {
            if let Some(value) = value {
                *context = value
            }
        })
    }
}

pub struct AssetsCollection;

impl<'c> TasksCollection<'c> for AssetsCollection {
    type Context = ();

    type Target = Option<()>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Assets collection"
    }

    fn handle(_context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|_| ())
    }
}

pub struct JavaCollection;

impl<'c> TasksCollection<'c> for JavaCollection {
    type Context = ();

    type Target = ();

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Java collection"
    }

    fn handle(_context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|()| ())
    }
}

pub struct GameDownloadingCollection;

impl<'c> TasksCollection<'c> for GameDownloadingCollection {
    type Context = &'c mut ProfilesConfig;

    type Target = Option<ModdedProfile>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Game downloading collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|opt: Option<ModdedProfile>| {
            let Some(profile) = opt else {
                return;
            };

            // PANICS: It will never panic because the profile
            // cannot be downloaded if it doesn't exists
            let prof = context
                .profiles
                .iter_mut()
                .find(|prof| prof.profile.id == profile.profile.id)
                .unwrap();

            *prof = Arc::new(profile);
            context.update_config().report_error();
        })
    }
}

pub struct GameDeletionCollection;

impl<'c> TasksCollection<'c> for GameDeletionCollection {
    type Context = ();

    type Target = ();

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Game deletion collection"
    }

    fn handle(_context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|()| ())
    }
}

pub struct ProjectCollection;

impl<'c> TasksCollection<'c> for ProjectCollection {
    type Context = &'c mut Option<Project>;

    type Target = Option<Project>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Project collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|value| {
            if let Some(value) = value {
                *context = Some(value)
            }
        })
    }
}

pub struct ProjectVersionsCollection;

impl<'c> TasksCollection<'c> for ProjectVersionsCollection {
    type Context = &'c mut Vec<Arc<Version>>;

    type Target = Option<Vec<Version>>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Project collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|value: Option<Vec<Version>>| {
            if let Some(value) = value {
                context.extend(value.into_iter().map(Arc::new));
            }
        })
    }
}

pub struct DependenciesCollection;

impl<'c> TasksCollection<'c> for DependenciesCollection {
    type Context = &'c mut Vec<SimpleDependency>;

    type Target = Option<Vec<SimpleDependency>>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Dependencies collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|value| {
            if let Some(deps) = value {
                *context = deps
            }
        })
    }
}
