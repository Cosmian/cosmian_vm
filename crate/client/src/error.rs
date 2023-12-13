use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Base64Decode(#[from] base64::DecodeError),
    #[error("Connection Error")]
    Connection,
    #[error("{0}")]
    Default(String),
    #[error("DNSName Error")]
    DNSName,
    #[error(transparent)]
    HexParsing(#[from] hex::FromHexError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Not Supported: {0}")]
    NotSupported(String),
    #[error("REST Request Failed: {0}")]
    RequestFailed(String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("REST Response Failed: {0}")]
    ResponseFailed(String),
    #[error("ServerCertificate Error")]
    ServerCertificate,
    #[error("Unexpected Error: {0}")]
    Unexpected(String),
    #[error(transparent)]
    UrlParsing(#[from] url::ParseError),
}
