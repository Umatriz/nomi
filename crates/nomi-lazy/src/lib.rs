use std::{future::Future, marker::PhantomData};

use tokio::sync::OnceCell;

pub type LazyFutureAlias<T> = std::pin::Pin<std::boxed::Box<dyn Future<Output = T>>>;

pub enum Undefined {}
pub enum NotTry {}
pub enum Try {}

#[derive(Debug)]
pub struct Lazy<
    T,
    State = Undefined,
    Out = T,
    F = fn() -> LazyFutureAlias<Out>,
    Fut = LazyFutureAlias<Out>,
> where
    F: Fn() -> Fut,
    Fut: Future<Output = Out>,
{
    cell: OnceCell<T>,
    init: F,
    _marker: PhantomData<State>,
}

impl<T> Lazy<T> {
    pub const fn new<F, Fut>(init: F) -> Lazy<T, NotTry, T, F, Fut>
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

    pub const fn new_try<F, Fut, Out>(init: F) -> Lazy<T, Try, Out, F, Fut>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Out>,
    {
        Lazy {
            cell: OnceCell::const_new(),
            init,
            _marker: PhantomData,
        }
    }
}

impl<T, F, Fut> Lazy<T, NotTry, T, F, Fut>
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

impl<T, F, Fut> Lazy<T, Try, anyhow::Result<T>, F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    pub fn get(&self) -> Option<&T> {
        self.cell.get()
    }

    pub async fn get_or_try_init(&self) -> anyhow::Result<&T> {
        self.cell.get_or_try_init(&self.init).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static STATIC_STATE: Lazy<i32, NotTry> = Lazy::new(|| {
        Box::pin(async {
            let _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            1
        })
    });

    static TRY_STATE: Lazy<i32, Try, anyhow::Result<i32>> =
        Lazy::new_try(|| Box::pin(async { Ok(1) }));

    #[tokio::test]
    async fn static_test() {
        let t = STATIC_STATE.get_or_init().await;

        assert_eq!(t, &1 as &i32);
    }

    #[tokio::test]
    async fn try_test() {
        let t = TRY_STATE.get_or_try_init().await;

        assert_eq!(t.unwrap(), &1 as &i32);
    }
}
