use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use super::{
    collection::{CollectionData, TasksCollection},
    Task,
};

#[derive(Default)]
pub struct TasksManager<'c> {
    collections: HashMap<TypeId, CollectionData<'c>>,
}

impl TasksManager<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'c> TasksManager<'c> {
    fn get_collection_mut<C>(&mut self) -> &mut CollectionData<'c>
    where
        C: TasksCollection<'c> + 'static,
    {
        self.collections
            .get_mut(&TypeId::of::<C>())
            .unwrap_or_else(move || {
                panic!(
                    "You must add `{}` collection to the `TaskManager` by calling `add_collection`",
                    type_name::<C>()
                )
            })
    }

    pub fn add_collection<C>(&mut self, context: C::Context) -> &mut Self
    where
        C: TasksCollection<'c> + 'static,
        C::Executor: Default + 'static,
    {
        self.collections.insert(
            TypeId::of::<C>(),
            CollectionData::from_collection::<C>(context),
        );
        self
    }

    pub fn listen_collection<C>(&mut self)
    where
        C: TasksCollection<'c> + 'static,
    {
        self.get_collection_mut::<C>().listen_all()
    }

    pub fn push_task<C>(&mut self, task: Task<C::Target>)
    where
        C: TasksCollection<'c> + 'static,
        C::Target: Send + 'static,
    {
        self.get_collection_mut::<C>().push_task::<C>(task);
    }
}
