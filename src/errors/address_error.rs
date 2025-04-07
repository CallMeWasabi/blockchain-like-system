use axum::http::StatusCode;
use super::error::{ErrorResponse, IntoErrorResponse};

pub enum APIAddressError {
    AddressNotFound(String),
    InvaidSignature(String),
    GenerateAddressError(String),
    AddressAlreadyExists(String),
    FindAddressError(String),
    UpdateBalanceError(String),
}

impl IntoErrorResponse for APIAddressError {
    fn error(&self) -> super::error::ErrorResponse {
        return match self {
            Self::AddressNotFound(addr) => ErrorResponse {
                error: format!("not found address: {}", addr),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::InvaidSignature(addr) => ErrorResponse {
                error: format!("invalid signature for address: {}", addr),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::GenerateAddressError(e) => ErrorResponse {
                error: format!("error while generating new address: {:?}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::AddressAlreadyExists(addr) => ErrorResponse {
                error: format!("address {} is already exist", addr),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::FindAddressError(e) => ErrorResponse {
                error: format!("find address error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::UpdateBalanceError(e) => ErrorResponse {
                error: format!("error while update address balance: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
        };
    }
}
