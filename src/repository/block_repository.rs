use async_trait::async_trait;
use bson::{Document, doc, from_document, oid::ObjectId};
use mockall::automock;
use mongodb::Database;
use std::sync::Arc;
use tracing::error;

use crate::entities::block_entity::BlockEntity;

pub type SharedBlockRepository = Arc<dyn BlockRepository + Send + Sync>;

#[async_trait]
#[automock]
pub trait BlockRepository {
    async fn find_latest(&self) -> Result<Option<BlockEntity>, String>;
    async fn find_by_hash(&self, hash: String) -> Result<Option<BlockEntity>, String>;

    async fn insert(&self, block: BlockEntity) -> Result<ObjectId, String>;

    async fn is_chain_valid(&self) -> Result<(), String>;
    async fn get_last_index(&self) -> Result<u64, String>;
}

pub struct MongoBlockRepository {
    db: Database,
}

impl MongoBlockRepository {
    pub fn creation(db: Database) -> SharedBlockRepository {
        return Arc::new(Self { db });
    }
}

#[async_trait]
impl BlockRepository for MongoBlockRepository {
    async fn find_latest(&self) -> Result<Option<BlockEntity>, String> {
        let doc = match self
            .db
            .collection::<Document>("blocks")
            .find_one(doc! {})
            .sort(doc! {
                "index": -1,
            })
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                error!("find block latest not found");
                return Ok(None);
            }
            Err(e) => {
                error!("find block latest error: {}", e);
                return Err(e.to_string());
            }
        };

        let block = from_document(doc).map_err(|e| {
            error!("convert doc to BlockEntity failed: {}", e);
            return e.to_string();
        })?;

        return Ok(Some(block));
    }

    async fn find_by_hash(&self, hash: String) -> Result<Option<BlockEntity>, String> {
        let doc = match self
            .db
            .collection::<Document>("blocks")
            .find_one(doc! {
                "hash": hash,
            })
            .await
        {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                error!("find block by hash not found");
                return Ok(None);
            }
            Err(e) => {
                error!("find block by hash error: {}", e);
                return Err(e.to_string());
            }
        };

        let block = from_document(doc).map_err(|e| {
            error!("convert doc to BlockEntity failed: {}", e);
            return e.to_string();
        })?;

        return Ok(Some(block));
    }

    async fn insert(&self, block: BlockEntity) -> Result<ObjectId, String> {
        let inserted_object_id = self
            .db
            .collection::<Document>("blocks")
            .insert_one(doc! {
                "index": block.index as i64,
                "timestamp": block.timestamp,
                "transactions": block.transactions,
                "previous_hash": block.previous_hash,
                "hash": block.hash,
                "nonce": block.nonce as i64
            })
            .await
            .map_err(|e| {
                error!("insert a new block failed: {}", e);
                return e.to_string();
            })?
            .inserted_id
            .as_object_id();

        return match inserted_object_id {
            Some(id) => Ok(id),
            None => {
                error!("issue with new _id");
                return Err(String::new());
            }
        };
    }

    async fn get_last_index(&self) -> Result<u64, String> {
        let result = self
            .db
            .collection("blocks")
            .find_one(doc! {})
            .sort(doc! { "index": -1 })
            .await
            .map_err(|e| {
                error!("get_last_index error: {}", e);
                e.to_string()
            })?;

        if let Some(doc) = result {
            let block: BlockEntity = from_document(doc).map_err(|e| {
                error!("convert doc to BlockEntity failed: {}", e);
                e.to_string()
            })?;
            Ok(block.index)
        } else {
            Ok(0)
        }
    }

    async fn is_chain_valid(&self) -> Result<(), String> {
        let mut cursor = self
            .db
            .collection::<Document>("blocks")
            .find(doc! {})
            .await
            .map_err(|e| {
                error!("is_chain_valid: failed to query blocks: {}", e);
                e.to_string()
            })?;

        let mut blocks: Vec<BlockEntity> = Vec::new();
        while cursor.advance().await.map_err(|e| e.to_string())? {
            let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
            let block: BlockEntity = from_document(doc).map_err(|e| e.to_string())?;
            blocks.push(block);
        }

        blocks.sort_by_key(|b| b.index);

        for i in 1..blocks.len() {
            if blocks[i].previous_hash != blocks[i - 1].hash {
                error!(
                    "Invalid chain at block index {}: previous_hash mismatch",
                    blocks[i].index
                );
                return Err(format!("Chain broken at block index {}", blocks[i].index));
            }
        }

        Ok(())
    }
}
