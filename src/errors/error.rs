use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug)]
pub struct ErrorResponse {
    pub error: String,
    pub status_code: StatusCode,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        return (
            self.status_code,
            Json(json!({
                "error": self.error
            })),
        )
            .into_response();
    }
}

pub trait IntoErrorResponse {
    fn error(&self) -> ErrorResponse;
}