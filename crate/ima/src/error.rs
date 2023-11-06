use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),
    #[error("{0}")]
    ParsingError(String),
    #[error("{0}")]
    NotImplemented(String),
    #[error("{0}")]
    ImaParsingError(String),
    #[error(transparent)]
    IntParsingError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    HexParsingError(#[from] hex::FromHexError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
