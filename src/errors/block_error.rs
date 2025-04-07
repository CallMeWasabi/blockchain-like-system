use super::error::{ErrorResponse, IntoErrorResponse};
use axum::http::StatusCode;

pub enum APIBlockError {
    InsertBlockError(String),
    FindBlockError(String),
    NotFound(String),
    InvalidChain(String),
}

impl IntoErrorResponse for APIBlockError {
    fn error(&self) -> ErrorResponse {
        match self {
            Self::InsertBlockError(msg) => ErrorResponse {
                error: format!("Insert block error: {}", msg),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::FindBlockError(msg) => ErrorResponse {
                error: format!("Find block error: {}", msg),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::NotFound(hash) => ErrorResponse {
                error: format!("Block not found with hash: {}", hash),
                status_code: StatusCode::NOT_FOUND,
            },
            Self::InvalidChain(reason) => ErrorResponse {
                error: format!("Blockchain invalid: {}", reason),
                status_code: StatusCode::BAD_REQUEST,
            },
        }
    }
}
