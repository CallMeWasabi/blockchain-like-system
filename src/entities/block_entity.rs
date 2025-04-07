use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::timer_helper::IntoTimerHelperShared;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct BlockEntity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<ObjectId>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl BlockEntity {
    pub fn new(
        index: u64,
        transactions: Vec<ObjectId>,
        previous_hash: String,
        hash: String,
        nonce: u64,
        t: IntoTimerHelperShared,
    ) -> Self {
        return Self {

            id: None,
            index,
            timestamp: t.now(),
            transactions,
            previous_hash,
            hash,
            nonce,
        }
    } }