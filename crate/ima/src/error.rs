use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Bincode(#[from] bincode::Error),
    #[error("{0}")]
    Parsing(String),
    #[error("{0}")]
    NotImplemented(String),
    #[error("{0}")]
    ImaParsing(String),
    #[error(transparent)]
    IntParsing(#[from] std::num::ParseIntError),
    #[error(transparent)]
    HexParsing(#[from] hex::FromHexError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
