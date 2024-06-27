use std::{
    any::{type_name, TypeId},
    collections::HashMap,
};

use eframe::egui::Ui;

use super::{
    collection::{CollectionData, TasksCollection},
    Task,
};

#[derive(Default)]
pub struct TasksManager {
    collections: HashMap<TypeId, CollectionData>,
}

impl TasksManager {
    pub fn ui(&self, ui: &mut Ui) {
        for collection in self.collections.values() {
            collection.ui(ui)
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    fn get_collection_mut<'c, C>(&mut self) -> &mut CollectionData
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

    pub fn add_collection<'c, C>(&mut self) -> &mut Self
    where
        C: TasksCollection<'c> + 'static,
        C::Executor: Default + 'static,
    {
        self.collections
            .insert(TypeId::of::<C>(), CollectionData::from_collection::<C>());
        self
    }

    pub fn handle_collection<'c, C>(&mut self, context: C::Context)
    where
        C: TasksCollection<'c> + 'static,
    {
        let handle = C::handle(context).into_any();
        self.get_collection_mut::<C>().handle_all(handle)
    }

    pub fn push_task<'c, C>(&mut self, task: Task<C::Target>)
    where
        C: TasksCollection<'c> + 'static,
        C::Target: Send + 'static,
    {
        self.get_collection_mut::<C>().push_task::<C>(task);
    }
}
