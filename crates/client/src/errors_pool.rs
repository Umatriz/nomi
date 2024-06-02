use std::{
    fmt::{Debug, Display},
    sync::{Arc, RwLock},
};

use once_cell::sync::Lazy;
use tracing::error;

pub static ERRORS_POOL: Lazy<Arc<RwLock<ErrorsPool>>> =
    Lazy::new(|| Arc::new(RwLock::new(ErrorsPool::default())));

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

#[derive(Default)]
pub struct ErrorsPoolState {
    pub is_window_open: bool,
    pub number_of_errors: usize,
}

pub trait ErrorPoolExt<T> {
    fn report_error(self) -> Option<T>;
}

impl<T, E> ErrorPoolExt<T> for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn report_error(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                if let Ok(mut pool) = ERRORS_POOL
                    .clone()
                    .write()
                    .inspect_err(|err| error!("Unable to write into the `ERRORS_POOL`\n{}", err))
                {
                    pool.push_error(error);
                }
                None
            }
        }
    }
}
