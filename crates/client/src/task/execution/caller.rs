use std::{any::Any, future::Future};

use super::{progress::TaskProgressShared, PinnedFuture};

pub enum Caller<T> {
    Standard(PinnedFuture<T>),
    Progressing(Box<dyn FnOnce(TaskProgressShared) -> PinnedFuture<T>>),
}

impl<T> Caller<T> {
    pub fn standard<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = T> + Send + 'static,
    {
        Self::Standard(Box::pin(fut))
    }

    pub fn progressing<F, Fut>(fun: F) -> Self
    where
        F: FnOnce(TaskProgressShared) -> Fut + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        Self::Progressing(Box::new(|progress| Box::pin((fun)(progress))))
    }
}

impl<T: Send + 'static> Caller<T> {
    pub fn into_any(self) -> Caller<Box<dyn Any + Send>> {
        match self {
            Self::Standard(fut) => Caller::standard(Box::pin(async move {
                Box::new(fut.await) as Box<dyn Any + Send>
            })),
            Self::Progressing(fun) => {
                let fun = Box::new(|progress| {
                    let fut = (fun)(progress);
                    Box::pin(async move { Box::new(fut.await) as Box<dyn Any + Send> })
                });

                Caller::progressing(fun)
            }
        }
    }
}
