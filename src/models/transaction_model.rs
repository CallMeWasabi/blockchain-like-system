use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub signature: String,
}