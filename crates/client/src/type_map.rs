use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Default)]
pub struct TypeMap {
    inner: HashMap<TypeId, Box<dyn Any>>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<T: Any>(mut self, value: T) -> Self {
        self.inner.insert(TypeId::of::<T>(), Box::new(value));
        self
    }

    pub fn insert<T: Any>(&mut self, values: T) {
        self.inner.insert(TypeId::of::<T>(), Box::new(values));
    }

    pub fn get<T: Any>(&self) -> Option<&T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.inner
            .get_mut(&TypeId::of::<T>())
            .and_then(|value| value.downcast_mut())
    }

    pub fn get_owned<T: Any + Clone>(&self) -> Option<T> {
        self.inner
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
            .cloned()
    }
}
