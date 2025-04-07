use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::timer_helper::IntoTimerHelperShared;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AddressEntity {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub public_key: String, // address is a wallet
    pub balance: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AddressEntity {
    pub fn new(public_key: String, t: IntoTimerHelperShared) -> Self {
        return Self {
            id: None,
            public_key,
            balance: 0,
            created_at: t.now(),
            updated_at: t.now(),
        };
    }
}
