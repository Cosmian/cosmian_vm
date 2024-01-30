use actix_web::{error::ResponseError, http::StatusCode, HttpResponse, HttpResponseBuilder};
use thiserror::Error;
use uuid::Uuid;

pub type ResponseWithError<T> = Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    BadRequest(String),
    #[error("Please update the cosmian_vm cli to version {0}")]
    BadUserAgent(String),
    #[error("{0}")]
    Certificate(String),
    #[error("A snapshot is currently processing (hold on before processing other actions)")]
    SnapshotIsProcessing,
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
            Self::Certificate(_)
            | Self::Command(_)
            | Self::Configuration(_)
            | Self::Cryptography(_)
            | Self::HexParsing(_)
            | Self::Ima(_)
            | Self::IO(_)
            | Self::Rustls(_)
            | Self::Serialization(_)
            | Self::TeeAttestation(_)
            | Self::Tpm(_)
            | Self::Unexpected(_)
            | Self::WalkDir(_) => StatusCode::INTERNAL_SERVER_ERROR,

            Self::SnapshotIsProcessing => StatusCode::CONFLICT,

            Self::BadRequest(_) | Self::BadUserAgent(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => {
                let error_uuid = Uuid::new_v4();
                tracing::error!(error = ?self, "[{error_uuid}] {}", self.to_string());
                HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(
                    format!("Something went wrong from the cosmian_vm agent. See cosmian_vm_agent logs for additional information. (error id: {error_uuid})"))
            }
            status_code => HttpResponseBuilder::new(status_code).body(self.to_string()),
        }
    }
}
