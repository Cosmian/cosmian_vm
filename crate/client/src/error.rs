use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Default(String),

    #[error(transparent)]
    HexParsingError(#[from] hex::FromHexError),

    #[error("Not Supported: {0}")]
    NotSupported(String),

    #[error("REST Request Failed: {0}")]
    RequestFailed(String),

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("REST Response Failed: {0}")]
    ResponseFailed(String),

    #[error("Unexpected Error: {0}")]
    UnexpectedError(String),

    #[error(transparent)]
    UrlParsing(#[from] url::ParseError),
}
