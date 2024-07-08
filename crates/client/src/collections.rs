use std::{marker::PhantomData, sync::Arc};

use egui_task_manager::*;
use nomi_core::{configs::profile::VersionProfile, repository::fabric_meta::FabricVersions};

use crate::{errors_pool::ErrorPoolExt, views::ProfilesConfig};

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

    type Target = Option<VersionProfile>;

    type Executor = executors::Linear;

    fn name() -> &'static str {
        "Game downloading collection"
    }

    fn handle(context: Self::Context) -> Handler<'c, Self::Target> {
        Handler::new(|opt: Option<VersionProfile>| {
            let Some(profile) = opt else {
                return;
            };

            // PANICS: It will never panic because the profile
            // cannot be downloaded if it doesn't exists
            let prof = context
                .profiles
                .iter_mut()
                .find(|prof| prof.id == profile.id)
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

// pub struct ModdingCollection<T, Ctx> {
//     _marker: PhantomData<(T, Ctx)>,
// }

// impl<'c, T, Ctx> TasksCollection<'c> for ModdingCollection<T, Ctx>
// where
//     T: Send + 'static,
//     Ctx: 'c,
// {
//     type Context = Ctx;

//     type Target = T;

//     type Executor = executors::Parallel;

//     fn name() -> &'static str {
//         "Modding collection"
//     }

//     fn handle(context: Self::Context) -> Handler<'c, Self::Target> {}
// }
