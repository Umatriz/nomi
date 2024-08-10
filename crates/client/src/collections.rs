use std::{collections::HashSet, sync::Arc};

use egui_task_manager::*;
use nomi_core::repository::fabric_meta::FabricVersions;
use nomi_modding::modrinth::{
    project::{Project, ProjectId},
    version::Version,
};

use crate::{
    errors_pool::ErrorPoolExt,
    views::{InstancesConfig, SimpleDependency},
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
    type Context = &'c InstancesConfig;

    type Target = Option<()>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Game downloading collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|_| {
            context.update_config_sync().report_error();
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
    type Context = (&'c mut Vec<SimpleDependency>, Option<&'c ProjectId>);

    type Target = Option<Vec<SimpleDependency>>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Dependencies collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(move |value| {
            if let Some(deps) = value {
                context.0.extend(deps);
                context.0.sort();
                context.0.dedup();
                if let Some(id) = context.1 {
                    context.0.retain(|d| d.project_id != *id);
                }
            }
        })
    }
}

pub struct ModsDownloadingCollection;

impl<'c> TasksCollection<'c> for ModsDownloadingCollection {
    type Context = &'c InstancesConfig;

    type Target = Option<()>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Mods downloading collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|_| {
            context.update_config_sync().report_error();
        })
    }
}

pub struct GameRunnerCollection;

impl<'c> TasksCollection<'c> for GameRunnerCollection {
    type Context = ();

    type Target = Option<()>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Game runner collection"
    }

    fn handle(_context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|_| ())
    }
}

pub struct DownloadAddedModsCollection;

impl<'c> TasksCollection<'c> for DownloadAddedModsCollection {
    type Context = (&'c mut HashSet<ProjectId>, &'c InstancesConfig);

    type Target = ProjectId;

    type Executor = executors::Parallel;

    fn name() -> &'static str {
        "Download added mod collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|id| {
            context.0.remove(&id);
            context.1.update_config_sync().report_error();
        })
    }
}
