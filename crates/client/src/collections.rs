// use crate::task::{executors, Handle, TasksCollection};

// pub struct AssetsCollection;

// impl<'c> TasksCollection<'c> for AssetsCollection {
//     type Context = ();

//     type Target = ();

//     type Executor = executors::Linear;

//     fn name() -> &'static str {
//         "Assets"
//     }

//     fn handle(_context: Self::Context) -> crate::task::Handle<'c, Self::Target> {
//         Handle::new(|()| ())
//     }
// }

// pub struct JavaCollection;

// impl<'c> TasksCollection<'c> for JavaCollection {
//     type Context = ();

//     type Target = ();

//     type Executor = executors::Linear;

//     fn name() -> &'static str {
//         "Java"
//     }

//     fn handle(context: Self::Context) -> Handle<'c, Self::Target> {
//         todo!()
//     }
// }
