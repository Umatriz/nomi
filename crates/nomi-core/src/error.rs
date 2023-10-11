#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No such version")]
    NoSuchVersion,
    #[error("Bad request")]
    BadRequest,
}
