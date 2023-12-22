use actix_web::{error::ResponseError, http::StatusCode, HttpResponse, HttpResponseBuilder};
use thiserror::Error;
use uuid::Uuid;

pub type ResponseWithError<T> = Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Certificate(String),
    #[error("{0}")]
    Command(String),
    #[error("{0}")]
    Configuration(String),
    #[error("{0}")]
    Cryptography(String),
    #[error(transparent)]
    HexParsing(#[from] hex::FromHexError),
    #[error(transparent)]
    Ima(#[from] ima::error::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Rustls(#[from] rustls::Error),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    TeeAttestation(#[from] tee_attestation::error::Error),
    #[error(transparent)]
    Tpm(#[from] tpm_quote::error::Error),
    #[error("{0}")]
    Unexpected(String),
    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Certificate(_)
            | Error::Command(_)
            | Error::Configuration(_)
            | Error::Cryptography(_)
            | Error::HexParsing(_)
            | Error::Ima(_)
            | Error::IO(_)
            | Error::Rustls(_)
            | Error::Serialization(_)
            | Error::TeeAttestation(_)
            | Error::Tpm(_)
            | Error::Unexpected(_)
            | Error::WalkDir(_) => StatusCode::INTERNAL_SERVER_ERROR,

            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => {
                let error_uuid = Uuid::new_v4();
                tracing::error!(error = ?self, "[{error_uuid}] {}", self.to_string());
                HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(
                    format!("Something went wrong from the cosmian_vm agent. See cosmian_vm logs for additional information. (error id: {error_uuid})"))
            }
            status_code => HttpResponseBuilder::new(status_code).body(self.to_string()),
        }
    }
}
