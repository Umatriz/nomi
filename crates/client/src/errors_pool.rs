use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use once_cell::sync::Lazy;
use parking_lot::RwLock;

pub static ERRORS_POOL: Lazy<Arc<RwLock<ErrorsPool>>> = Lazy::new(|| Arc::new(RwLock::new(ErrorsPool::default())));

pub trait Error: Display + Debug {}

impl<T: Display + Debug> Error for T {}

#[derive(Default)]
pub struct ErrorsPool {
    errors: Vec<Arc<dyn Error + Send + Sync>>,
}

impl ErrorsPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_error<E>(&mut self, error: E)
    where
        E: Error + Send + Sync + 'static,
    {
        self.errors.push(Arc::new(error))
    }

    pub fn iter_errors(&self) -> impl Iterator<Item = Arc<dyn Error + Send + Sync + '_>> {
        self.errors.iter().cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn clear(&mut self) {
        self.errors = vec![]
    }
}

pub trait ErrorPoolExt<T> {
    fn report_error(self) -> Option<T>;
    fn report_error_with_context<C>(self, context: C) -> Option<T>
    where
        C: Display + Send + Sync + 'static;
}

impl<T, E> ErrorPoolExt<T> for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn report_error(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                let mut pool = ERRORS_POOL.write();
                pool.push_error(error);
                None
            }
        }
    }

    fn report_error_with_context<C>(self, context: C) -> Option<T>
    where
        C: Display + Send + Sync + 'static,
    {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                let mut pool = ERRORS_POOL.write();
                pool.push_error(anyhow::Error::msg(error).context(context));
                None
            }
        }
    }
}
