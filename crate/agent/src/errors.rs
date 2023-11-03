use actix_web::{error::ResponseError, http::StatusCode, HttpResponse, HttpResponseBuilder};
use thiserror::Error;
use uuid::Uuid;

pub type ResponseWithError<T> = Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    CommandError(String),
    #[error("{0}")]
    ImaParsingError(String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    WalkDirError(#[from] walkdir::Error),
    #[error(transparent)]
    IntParsingError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    HexParsingError(#[from] hex::FromHexError),
    #[error(transparent)]
    TeeAttestation(#[from] tee_attestation::error::Error),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::IOError(_)
            | Error::CommandError(_)
            | Error::WalkDirError(_)
            | Error::ImaParsingError(_)
            | Error::IntParsingError(_)
            | Error::TeeAttestation(_)
            | Error::HexParsingError(_) => StatusCode::INTERNAL_SERVER_ERROR,

            Error::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => {
                let error_uuid = Uuid::new_v4();
                tracing::error!(error = ?self, "[{error_uuid}] {}", self.to_string());
                HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).body(
                    format!("Something went wrong from our side. We were notified of the problem but you can contact us to provide additional information about your workflow. (error id: {error_uuid})"))
            }
            status_code => HttpResponseBuilder::new(status_code).body(self.to_string()),
        }
    }
}
