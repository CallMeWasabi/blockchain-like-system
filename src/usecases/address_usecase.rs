use std::sync::Arc;

use crate::{
    entities::address_entity::AddressEntity,
    errors::{address_error::APIAddressError, error::IntoErrorResponse},
    models::address_model::{CoinWithAddress, InsertAddress},
    repository::address_repository::SharedAddressRepository,
    timer_helper::IntoTimerHelperShared,
};
use bson::oid::ObjectId;

pub struct AddressUsecase {
    address_repository: SharedAddressRepository,
    timer_helper: IntoTimerHelperShared,
}

impl AddressUsecase {
    pub fn creation(
        address_repository: SharedAddressRepository,
        timer_helper: IntoTimerHelperShared,
    ) -> Arc<Self> {
        return Arc::new(Self {
            address_repository,
            timer_helper,
        });
    }

    pub async fn create_new_address(
        &self,
        insert_address: InsertAddress,
    ) -> Result<ObjectId, Box<dyn IntoErrorResponse>> {
        match self
            .address_repository
            .get_by_address(insert_address.public_key.clone())
            .await
        {
            Ok(Some(_)) => {
                return Err(Box::new(APIAddressError::AddressAlreadyExists(
                    insert_address.public_key,
                )));
            }
            _ => {}
        };

        return match self
            .address_repository
            .insert(AddressEntity::new(
                insert_address.public_key,
                Arc::clone(&self.timer_helper),
            ))
            .await
        {
            Ok(id) => Ok(id),
            Err(e) => Err(Box::new(APIAddressError::GenerateAddressError(e))),
        };
    }

    pub async fn deposit_coin(
        &self,
        coin_with_address: CoinWithAddress,
    ) -> Result<(), Box<dyn IntoErrorResponse>> {
        return match self
            .address_repository
            .deposit(
                AddressEntity::new(coin_with_address.public_key, Arc::clone(&self.timer_helper)),
                coin_with_address.amount,
            )
            .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(APIAddressError::UpdateBalanceError(e))),
        };
    }

    pub async fn withdraw_coin(
        &self,
        coin_with_address: CoinWithAddress,
    ) -> Result<(), Box<dyn IntoErrorResponse>> {
        return match self
            .address_repository
            .withdraw(
                AddressEntity::new(coin_with_address.public_key, Arc::clone(&self.timer_helper)),
                coin_with_address.amount,
            )
            .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(APIAddressError::UpdateBalanceError(e))),
        };
    }
}
