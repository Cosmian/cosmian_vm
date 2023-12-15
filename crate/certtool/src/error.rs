use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Base64Decode(#[from] acme_lib::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Crypto(#[from] openssl::error::ErrorStack),
    #[error(transparent)]
    TeeAttestation(#[from] tee_attestation::error::Error),
}
