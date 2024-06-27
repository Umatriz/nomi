use std::any::Any;

pub type AnyHandle<'h> = Handle<'h, Box<dyn Any + Send>>;

pub struct Handle<'h, T>(Box<dyn FnMut(T) + 'h>);

impl<'h, T: 'static> Handle<'h, T> {
    pub fn new(handle: impl FnMut(T) + 'h) -> Self {
        Self(Box::new(handle))
    }

    pub(in crate::task) fn into_any(mut self) -> AnyHandle<'h> {
        let handle = Box::new(move |boxed_any: Box<dyn Any + Send>| {
            let reference = boxed_any.downcast::<T>().unwrap();
            (self.0)(*reference)
        });

        Handle(handle)
    }

    pub fn apply(&mut self, value: T) {
        (self.0)(value)
    }
}
