use crate::timer_helper::IntoTimerHelperShared;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Rejected,
    Invalid,
    Expired,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TransactionEntity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub block_hash: Option<String>,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub signature: String,
    pub timestamp: i64,
    pub status: TransactionStatus,
}

impl TransactionEntity {
    pub fn new(
        from: String,
        to: String,
        amount: u64,
        signature: String,
        status: TransactionStatus,
        t: IntoTimerHelperShared,
    ) -> Self {
        return Self {
            id: None,
            block_hash: None,
            from,
            to,
            amount,
            signature,
            timestamp: t.now(),
            status,
        };
    }
}
