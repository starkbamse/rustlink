use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not find symbol")]
    NotFound,
    #[error("Could not deserialize binary data")]
    Deserialize,
    #[error("Database internal error: {0}")]
    SledError(#[from] sled::Error),
}