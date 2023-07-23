pub use crate::error::Error;

pub type Result<T> = core::result::Result<T, Error>;

/// Wrapper for newtype pattern
pub struct W<T>(pub T);
