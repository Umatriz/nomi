use std::{future::Future, marker::PhantomData};

use tokio::sync::OnceCell;

pub type LazyFutureAlias<T> = std::pin::Pin<std::boxed::Box<dyn Future<Output = T>>>;

pub struct Undefined;
pub struct NotTry;
pub struct Try;

pub struct LazyBuilder;

impl LazyBuilder {
    pub const fn new_not_try<T, F, Fut>(init: F) -> Lazy<T, NotTry, F, Fut>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = T>,
    {
        Lazy {
            cell: OnceCell::const_new(),
            init,
            _marker: PhantomData,
        }
    }

    pub const fn new_try<T, F, Fut>(init: F) -> Lazy<T, Try, F, Fut, anyhow::Result<T>>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = anyhow::Result<T>>,
    {
        Lazy {
            cell: OnceCell::const_new(),
            init,
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Lazy<T, State, F = fn() -> LazyFutureAlias<T>, Fut = LazyFutureAlias<T>, Out = T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Out>,
{
    cell: OnceCell<T>,
    init: F,
    _marker: PhantomData<State>,
}

impl<T, F, Fut> Lazy<T, NotTry, F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
{
    pub fn get(&self) -> Option<&T> {
        self.cell.get()
    }

    pub async fn get_or_init(&self) -> &T {
        self.cell.get_or_init(&self.init).await
    }
}

impl<T, F, Fut> Lazy<T, Try, F, Fut, anyhow::Result<T>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    pub fn get(&self) -> Option<&T> {
        self.cell.get()
    }

    pub async fn get_or_init(&self) -> anyhow::Result<&T> {
        self.cell.get_or_try_init(&self.init).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static STATE: Lazy<i32, NotTry> = LazyBuilder::new_not_try(|| {
        Box::pin(async {
            let _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            1
        })
    });

    #[tokio::test]
    async fn init_test() {
        let t = STATE.get_or_init().await;

        assert_eq!(t, &1 as &i32);
    }
}
