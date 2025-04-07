use serde::{Deserialize, Serialize};


pub struct InsertAddress {
    pub public_key: String
}

#[derive(Serialize, Deserialize)]
pub struct CoinWithAddress {
    pub public_key: String,
    pub amount: u64,
}


