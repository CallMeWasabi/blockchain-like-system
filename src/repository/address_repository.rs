use std::sync::Arc;

use crate::entities::address_entity::AddressEntity;
use async_trait::async_trait;
use bson::{Document, doc, from_document, oid::ObjectId};
use mockall::automock;
use mongodb::Database;
use tracing::error;

pub type SharedAddressRepository = Arc<dyn AddressRepository + Send + Sync>;

#[async_trait]
#[automock]
pub trait AddressRepository {
    async fn get_by_id(&self, id: ObjectId) -> Result<Option<AddressEntity>, String>;
    async fn get_by_address(&self, address: String) -> Result<Option<AddressEntity>, String>;
    async fn insert(&self, insert_address: AddressEntity) -> Result<ObjectId, String>;
    async fn deposit(&self, address: AddressEntity, amount: u64) -> Result<(), String>;
    async fn withdraw(&self, address: AddressEntity, amount: u64) -> Result<(), String>;
}

pub struct MongoAddressRepository {
    db: Database,
}

impl MongoAddressRepository {
    pub fn creation(db: Database) -> SharedAddressRepository {
        return Arc::new(Self { db });
    }
}

#[async_trait]
impl AddressRepository for MongoAddressRepository {
    async fn get_by_id(&self, id: ObjectId) -> Result<Option<AddressEntity>, String> {
        let result = self
            .db
            .collection::<Document>("addresses")
            .find_one(doc! {
                "_id": id,
            })
            .await;

        let doc = match result {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                error!("find address by id not found");
                return Ok(None);
            }
            Err(e) => {
                error!("find address by id error: {}", e);
                return Err(e.to_string());
            }
        };

        let address_entity: AddressEntity = match from_document(doc) {
            Ok(data) => data,
            Err(e) => {
                error!("convert doc to AddressEntity failed: {}", e);
                return Err(e.to_string());
            }
        };

        return Ok(Some(address_entity));
    }

    async fn get_by_address(&self, address: String) -> Result<Option<AddressEntity>, String> {
        let result = self
            .db
            .collection::<Document>("addresses")
            .find_one(doc! {
                "public_key": address,
            })
            .await;

        let doc = match result {
            Ok(Some(doc)) => doc,
            Ok(None) => {
                error!("find by address not found");
                return Ok(None);
            }
            Err(e) => {
                error!("find address by address: {}", e);
                return Err(e.to_string());
            }
        };

        let address_entity: AddressEntity = match from_document(doc) {
            Ok(data) => data,
            Err(e) => {
                error!("convert doc to AddressEntity failed: {}", e);
                return Err(e.to_string());
            }
        };

        return Ok(Some(address_entity));
    }

    async fn insert(&self, new_address: AddressEntity) -> Result<ObjectId, String> {
        let result = self
            .db
            .collection::<Document>("addresses")
            .insert_one(doc! {
                "public_key": new_address.public_key,
                "balance": new_address.balance as i64,
                "created_at": new_address.created_at,
                "updated_at": new_address.updated_at,
            })
            .await;

        let inserted_address = match result {
            Ok(doc) => doc,
            Err(e) => {
                error!("insert a new address failed: {}", e);
                return Err(e.to_string());
            }
        };

        let inserted_object_id = match inserted_address.inserted_id.as_object_id() {
            Some(id) => id,
            None => {
                error!("issue with new _id");
                return Err(String::new());
            }
        };

        return Ok(inserted_object_id);
    }

    async fn deposit(&self, address: AddressEntity, amount: u64) -> Result<(), String> {
        let filter = doc! {
            "public_key": &address.public_key
        };
        let update = doc! {
            "$inc": { "balance": amount as i64 },
            "$set": { "updated_at": address.updated_at }
        };

        let result = self
            .db
            .collection::<Document>("addresses")
            .update_one(filter, update)
            .await;

        return match result {
            Ok(update_result) => {
                if update_result.matched_count == 0 {
                    error!("deposit: address not found: {}", address.public_key);
                    return Err("address not found".to_string());
                }

                Ok(())
            }
            Err(e) => {
                error!("deposit error: {}", e);
                Err(e.to_string())
            }
        };
    }

    async fn withdraw(&self, address: AddressEntity, amount: u64) -> Result<(), String> {
        let filter = doc! {
            "public_key": &address.public_key,
        };
        let update = doc! {
            "$inc": { "balance": -1 * (amount as i64)},
            "$set": { "updated_at": address.updated_at }
        };

        let result = self
            .db
            .collection::<Document>("addresses")
            .update_one(filter, update)
            .await;

        return match result {
            Ok(update_result) => {
                if update_result.matched_count == 0 {
                    error!("withdraw: address not found: {}", address.public_key);
                    return Err("withdraw not found".to_string());
                }

                Ok(())
            }
            Err(e) => {
                error!("withdraw error: {}", e);
                Err(e.to_string())
            }
        };
    }
}
