use std::any::Any;

use crate::channel::Channel;

pub type AnyTaskHandle<'c> = TaskHandle<'c, Box<dyn Any + Send>>;

pub struct TaskHandle<'c, T> {
    channel: Channel<T>,
    handle: Box<dyn FnMut(T) + 'c>,
}

impl<'c, T: 'static> TaskHandle<'c, T> {
    pub fn new(handle: impl FnMut(T) + 'c) -> Self {
        Self {
            channel: Channel::new(100),
            handle: Box::new(handle),
        }
    }

    pub fn channel(&self) -> &Channel<T> {
        &self.channel
    }

    pub fn into_any(mut self) -> AnyTaskHandle<'c> {
        TaskHandle {
            channel: Channel::new(100),
            handle: Box::new(move |boxed_any| {
                let reference = boxed_any.downcast::<T>().unwrap();
                (self.handle)(*reference)
            }),
        }
    }

    pub fn handle(&mut self) {
        if let Ok(data) = self.channel.try_recv() {
            (self.handle)(data)
        }
    }
}
