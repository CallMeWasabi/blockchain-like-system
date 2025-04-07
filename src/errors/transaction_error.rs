use super::error::{ErrorResponse, IntoErrorResponse};
use axum::http::StatusCode;
use bson::oid::ObjectId;

pub enum APITransactionError {
    BalanceNotEnough(String, u64, u64),
    InvalidSignature,
    VerifySignatureError(String),
    InsertTransactionError(String),
    NotFound(ObjectId),
    FindError(String),
    UpdateStatusError(String),
}

impl IntoErrorResponse for APITransactionError {
    fn error(&self) -> super::error::ErrorResponse {
        return match self {
            Self::BalanceNotEnough(sender, current_balance, expected_balance) => ErrorResponse {
                error: format!(
                    "balance is not enough from sender {} need to send {} but have {}",
                    sender, expected_balance, current_balance
                ),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::InvalidSignature => ErrorResponse {
                error: format!("invalid signature"),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::VerifySignatureError(e) => ErrorResponse {
                error: format!("verify signature error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::InsertTransactionError(e) => ErrorResponse {
                error: format!("insert a new transaction error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::NotFound(tx_id) => ErrorResponse {
                error: format!("not found transaction id: {}", tx_id),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::FindError(e) => ErrorResponse {
                error: format!("find transaction error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::UpdateStatusError(e) => ErrorResponse {
                error: format!("Update transaction status error: {}", e),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
            },
        };
    }
}
