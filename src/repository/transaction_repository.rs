use std::sync::Arc;

use async_trait::async_trait;
use bson::{Document, doc, from_document, oid::ObjectId, to_bson};
use mockall::automock;
use mongodb::Database;
use tracing::error;

use crate::entities::transaction_entity::{TransactionEntity, TransactionStatus};

pub type SharedTransactionRepository = Arc<dyn TransactionRepository + Send + Sync>;

#[async_trait]
#[automock]
pub trait TransactionRepository {
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<TransactionEntity>, String>;
    async fn find_by_address(&self, address: String) -> Result<Vec<TransactionEntity>, String>;
    async fn find_all_pending(&self) -> Result<Vec<TransactionEntity>, String>;

    async fn insert(&self, tx: TransactionEntity) -> Result<ObjectId, String>;

    async fn update_status(&self, tx_id: ObjectId, status: TransactionStatus)
    -> Result<(), String>;
    async fn mark_confirmed(&self, tx_id: ObjectId, block_hash: String) -> Result<(), String>;
}

pub struct MongoTransactionRepository {
    db: Database,
}

impl MongoTransactionRepository {
    pub fn creation(db: Database) -> SharedTransactionRepository {
        return Arc::new(Self { db });
    }
}

#[async_trait]
impl TransactionRepository for MongoTransactionRepository {
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<TransactionEntity>, String> {
        let result = self
            .db
            .collection::<Document>("transactions")
            .find_one(doc! {
                "_id": id
            })
            .await;

        let doc = match result {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                error!("find tx by id not found");
                return Ok(None);
            }
            Err(e) => {
                error!("find tx by id error: {}", e);
                return Err(e.to_string());
            }
        };

        let tx_entity = from_document(doc).map_err(|e| {
            error!("convert doc to TransactionEntity failed: {}", e);
            return e.to_string();
        })?;

        return Ok(Some(tx_entity));
    }

    async fn find_by_address(&self, address: String) -> Result<Vec<TransactionEntity>, String> {
        let filter = doc! {
            "$or": [
                { "from": address.clone() },
                { "to": address.clone() }
            ]
        };
        let mut cursor = self
            .db
            .collection::<Document>("transactions")
            .find(filter)
            .await
            .map_err(|e| {
                error!("find tx by address error: {}", e);
                return e.to_string();
            })?;

        let mut txs = Vec::new();
        while cursor.advance().await.map_err(|e| {
            error!("find tx by address error: {}", e);
            return e.to_string();
        })? {
            let tx = from_document(cursor.deserialize_current().map_err(|e| {
                error!("failed to deserialize: {}", e);
                return e.to_string();
            })?)
            .map_err(|e| {
                error!("convert doc to TransactionEntity failed: {}", e);
                return e.to_string();
            })?;

            txs.push(tx);
        }

        return Ok(txs);
    }

    async fn find_all_pending(&self) -> Result<Vec<TransactionEntity>, String> {
        let filter = doc! { "status": "pending"};
        let mut cursor = self
            .db
            .collection::<Document>("transactions")
            .find(filter)
            .sort(doc! {"timestamp": 1})
            .await
            .map_err(|e| {
                error!("find tx status pending error: {}", e);
                return e.to_string();
            })?;

        let mut txs = Vec::new();
        while cursor.advance().await.map_err(|e| {
            error!("find tx status pending error: {}", e);
            return e.to_string();
        })? {
            let tx = from_document(cursor.deserialize_current().map_err(|e| {
                error!("failed to deserialize: {}", e);
                return e.to_string();
            })?)
            .map_err(|e| {
                error!("convert doc to TransactionEntity failed: {}", e);
                return e.to_string();
            })?;

            txs.push(tx);
        }

        return Ok(txs);
    }

    async fn insert(&self, tx: TransactionEntity) -> Result<ObjectId, String> {
        let inserted_object_id = self
            .db
            .collection::<Document>("transactions")
            .insert_one(doc! {
                "from": tx.from,
                "to": tx.to,
                "amount": tx.amount as i64,
                "signature": tx.signature,
                "timestamp": tx.timestamp,
                "status": to_bson(&tx.status).unwrap(),
            })
            .await
            .map_err(|e| {
                error!("insert a new transaction failed: {}", e);
                return e.to_string();
            })?
            .inserted_id
            .as_object_id();

        let object_id = match inserted_object_id {
            Some(id) => id,
            None => {
                error!("issue with new _id");
                return Err(String::new());
            }
        };

        return Ok(object_id);
    }

    async fn update_status(
        &self,
        tx_id: ObjectId,
        status: TransactionStatus,
    ) -> Result<(), String> {
        let filter = doc! { "_id": tx_id };
        let update = doc! {
            "$set": {
                "status": to_bson(&status).map_err(|e| e.to_string())?
            }
        };

        let result = self
            .db
            .collection::<Document>("transactions")
            .update_one(filter, update)
            .await
            .map_err(|e| e.to_string())?;

        if result.matched_count == 0 {
            return Err("Transaction not found".to_string());
        }

        Ok(())
    }

    async fn mark_confirmed(&self, tx_id: ObjectId, block_hash: String) -> Result<(), String> {
        let filter = doc! { "_id": tx_id };

        let update = doc! {
            "$set": {
                "status": to_bson(&TransactionStatus::Confirmed).map_err(|e| e.to_string())?,
                "block_hash": block_hash,
            }
        };

        let result = self
            .db
            .collection::<Document>("transactions")
            .update_one(filter, update)
            .await
            .map_err(|e| e.to_string())?;

        if result.matched_count == 0 {
            return Err("Transaction not found".to_string());
        }

        Ok(())
    }
}
