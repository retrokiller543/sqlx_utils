use std::borrow::Cow;
use thiserror::Error;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("An error occurred while executing query: {message}")]
    Repository { message: Cow<'static, str> },
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("Failed to acquire lock on mutex")]
    MutexLockError,
    #[error(transparent)]
    Boxed(Box<dyn std::error::Error + Send>),
}
