use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;

use crate::{Error, ErrorKind};

/// DTO for server error
#[derive(Serialize)]
struct ErrorResponse<'a> {
    /// HTTP error code
    code: u16,

    /// Kind of error
    #[serde(flatten)]
    error: &'a ErrorKind,
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self.kind() {
            ErrorKind::UnsupportedFileType => StatusCode::BAD_REQUEST,
            ErrorKind::FileTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            ErrorKind::EntityDeleted => StatusCode::FORBIDDEN,
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            ErrorKind::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();

        if self.is_internal() {
            tracing::error!(
                error = ?self.kind(),
                file = self.location.file(),
                line = self.location.line()
            );
        }
        let res = ErrorResponse {
            code: status.as_u16(),
            error: self.kind(),
        };

        HttpResponse::build(status).json(res)
    }
}
