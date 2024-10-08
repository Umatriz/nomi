use egui_task_manager::Progress;

#[async_trait::async_trait]
pub trait ProgressSender<P: Send>: Sync + Send {
    async fn update(&self, data: P);
}

#[async_trait::async_trait]
impl<P: Send> ProgressSender<P> for tokio::sync::mpsc::Sender<P> {
    async fn update(&self, data: P) {
        let _ = self.send(data).await;
    }
}

#[async_trait::async_trait]
impl<P: Send> ProgressSender<P> for std::sync::mpsc::Sender<P> {
    async fn update(&self, data: P) {
        let _ = self.send(data);
    }
}

pub struct MappedSender<I, T> {
    inner: Box<dyn ProgressSender<T>>,
    mapper: Box<dyn Fn(I) -> T + Sync + Send>,
    side_effect: Option<Box<dyn Fn() + Sync + Send>>,
}

#[async_trait::async_trait]
impl<I: Send, T: Send> ProgressSender<I> for MappedSender<I, T> {
    async fn update(&self, data: I) {
        let mapped = (self.mapper)(data);
        self.inner.update(mapped).await;

        if let Some(side_effect) = self.side_effect.as_ref() {
            (side_effect)();
        }
    }
}

impl<I, T> MappedSender<I, T> {
    pub fn new<F>(sender: Box<dyn ProgressSender<T>>, mapper: F) -> Self
    where
        F: Fn(I) -> T + Sync + Send + 'static,
    {
        Self {
            inner: sender,
            mapper: Box::new(mapper),
            side_effect: None,
        }
    }

    #[must_use]
    pub fn with_side_effect(mut self, side_effect: impl Fn() + Send + Sync + 'static) -> Self {
        self.side_effect = Some(Box::new(side_effect));
        self
    }
}

impl<I: Progress + 'static> MappedSender<I, Box<dyn Progress>> {
    pub fn new_progress_mapper(sender: Box<dyn ProgressSender<Box<dyn Progress>>>) -> Self {
        Self {
            inner: sender,
            mapper: Box::new(|value| Box::new(value) as Box<dyn Progress>),
            side_effect: None,
        }
    }

    pub fn new_progress_mapper_with_side_effect(
        sender: Box<dyn ProgressSender<Box<dyn Progress>>>,
        side_effect: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner: sender,
            mapper: Box::new(|value| Box::new(value) as Box<dyn Progress>),
            side_effect: Some(Box::new(side_effect)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;

    #[test]
    fn construction_test() {
        let (sender, _) = std::sync::mpsc::channel();
        let _ = MappedSender::new(Box::new(sender), |val: u32| Box::new(val) as Box<dyn Any + Send>);

        let (sender, _) = tokio::sync::mpsc::channel(1);
        let _ = MappedSender::new(Box::new(sender), |val: u32| Box::new(val) as Box<dyn Any + Send>);
    }
}
