use std::{future::Future, marker::PhantomData};

use tokio::sync::{Mutex, OnceCell};

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

/// Generic implementation of `Lazy`
impl<T, State, Out: 'static> Lazy<T, State, Out> {
    pub fn get(&self) -> Option<&T> {
        self.cell.get()
    }

    /// Wrapper over [`tokio::sync::OnceCell::set`]
    ///
    /// It doesn't mutates current value of `Lazy`
    /// for mutating see [`Lazy::get_mut`]
    pub fn set(&self, value: T) -> anyhow::Result<(), tokio::sync::SetError<T>> {
        self.cell.set(value)
    }

    /// Wrapper over [`tokio::sync::OnceCell::get_mut`]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.cell.get_mut()
    }

    /// Wrapper over [`tokio::sync::OnceCell::initialized`]
    pub fn initialized(&self) -> bool {
        self.cell.initialized()
    }

    /// Wrapper over [`tokio::sync::OnceCell::into_inner`]
    pub fn into_inner(self) -> Option<T> {
        self.cell.into_inner()
    }

    /// Wrapper over [`tokio::sync::OnceCell::into_inner`]
    ///
    /// Takes ownership of the current value, leaving the cell empty. Returns
    /// `None` if the cell is empty.
    pub fn take(&mut self) -> Option<T> {
        self.cell.take()
    }
}

/// Implementation of `Lazy` that can't return an error
impl<T, F, Fut> Lazy<T, NotTry, T, F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
{
    pub async fn get_or_init(&self) -> &T {
        self.cell.get_or_init(&self.init).await
    }
}

/// Implementation of `Lazy` that can return an error
impl<T, F, Fut> Lazy<T, Try, anyhow::Result<T>, F, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    pub async fn get_or_try_init(&self) -> anyhow::Result<&T> {
        self.cell.get_or_try_init(&self.init).await
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;

    use super::*;

    static STATIC_STATE: Lazy<i32, NotTry> = Lazy::new(|| Box::pin(async { 1 }));

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

    #[tokio::test]
    async fn get_test() {
        let s = STATIC_STATE.get();
        let t = TRY_STATE.get();

        assert_eq!(s, None);
        assert_eq!(t, None);
    }

    static MUTEX_BASED_LAZY: Mutex<Lazy<i32, Try, anyhow::Result<i32>>> =
        Mutex::const_new(Lazy::new_try(|| Box::pin(async { Ok(1) })));

    #[tokio::test]
    async fn get_mut_test() {
        let mut lock = MUTEX_BASED_LAZY.try_lock().unwrap();
        lock.get_or_try_init().await.unwrap();
        let data = lock.get_mut().unwrap();
        *data = 3;
        assert_eq!(lock.get(), Some(3).as_ref());
    }

    static ONCE_CELL: Mutex<OnceCell<i32>> = Mutex::const_new(OnceCell::const_new());

    #[tokio::test]
    async fn once_cell_get_mut_test() {
        let mut lock = ONCE_CELL.lock().await;
        lock.get_or_init(|| async { 1 }).await;
        let data = lock.get_mut().unwrap();
        *data = 2;

        assert_eq!(lock.get(), Some(2).as_ref());
    }
}
