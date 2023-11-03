use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("REST Request Failed: {0}")]
    RequestFailed(String),

    #[error("REST Response Failed: {0}")]
    ResponseFailed(String),

    #[error("Unexpected Error: {0}")]
    UnexpectedError(String),

    #[error("Not Supported: {0}")]
    NotSupported(String),

    #[error("{0}")]
    Default(String),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
