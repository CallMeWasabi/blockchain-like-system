use std::sync::Arc;

use crate::{
    crypto_helper,
    entities::transaction_entity::{TransactionEntity, TransactionStatus},
    errors::{
        address_error::APIAddressError, error::IntoErrorResponse,
        transaction_error::APITransactionError,
    },
    models::transaction_model::CreateTransactionRequest,
    repository::{
        address_repository::SharedAddressRepository,
        transaction_repository::SharedTransactionRepository,
    },
    timer_helper::IntoTimerHelperShared,
};
use bson::oid::ObjectId;

pub struct TransactionUsecase {
    tx_repo: SharedTransactionRepository,
    addr_repo: SharedAddressRepository,
    timer_helper: IntoTimerHelperShared,
}

impl TransactionUsecase {
    pub fn creation(
        tx_repo: SharedTransactionRepository,
        addr_repo: SharedAddressRepository,
        timer_helper: IntoTimerHelperShared,
    ) -> Arc<Self> {
        return Arc::new(Self {
            tx_repo,
            addr_repo,
            timer_helper,
        });
    }

    pub async fn get_by_id(
        &self,
        tx_id: ObjectId,
    ) -> Result<TransactionEntity, Box<dyn IntoErrorResponse>> {
        return match self.tx_repo.find_by_id(tx_id.clone()).await {
            Ok(Some(tx)) => Ok(tx),
            Ok(None) => Err(Box::new(APITransactionError::NotFound(tx_id))),
            Err(e) => Err(Box::new(APITransactionError::FindError(e))),
        };
    }

    pub async fn get_by_address(
        &self,
        address: String,
    ) -> Result<Vec<TransactionEntity>, Box<dyn IntoErrorResponse>> {
        match self.addr_repo.get_by_address(address.clone()).await {
            Ok(Some(_)) => {}
            Ok(None) => return Err(Box::new(APIAddressError::AddressNotFound(address))),
            Err(e) => return Err(Box::new(APIAddressError::FindAddressError(e))),
        };

        return match self.tx_repo.find_by_address(address).await {
            Ok(txs) => Ok(txs),
            Err(e) => return Err(Box::new(APITransactionError::FindError(e))),
        };
    }

    pub async fn get_all_pending(
        &self,
    ) -> Result<Vec<TransactionEntity>, Box<dyn IntoErrorResponse>> {
        return match self.tx_repo.find_all_pending().await {
            Ok(txs) => return Ok(txs),
            Err(e) => Err(Box::new(APITransactionError::FindError(e))),
        };
    }

    pub async fn confirm_transaction(
        &self,
        tx_id: ObjectId,
        block_hash: String,
    ) -> Result<(), Box<dyn IntoErrorResponse>> {
        return match self.tx_repo.mark_confirmed(tx_id.clone(), block_hash).await {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(APITransactionError::UpdateStatusError(e))),
        };
    }

    pub async fn reject_transaction(
        &self,
        tx_id: ObjectId,
    ) -> Result<(), Box<dyn IntoErrorResponse>> {
        return match self
            .tx_repo
            .update_status(tx_id.clone(), TransactionStatus::Rejected)
            .await
        {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(APITransactionError::UpdateStatusError(e))),
        };
    }

    pub async fn create_transaction(
        &self,
        req: CreateTransactionRequest,
    ) -> Result<ObjectId, Box<dyn IntoErrorResponse>> {
        let sender = match self.addr_repo.get_by_address(req.from.clone()).await {
            Ok(Some(sender)) => sender,
            Ok(None) => return Err(Box::new(APIAddressError::AddressNotFound(req.from))),
            Err(e) => return Err(Box::new(APIAddressError::FindAddressError(e))),
        };
        let _receiver: crate::entities::address_entity::AddressEntity =
            match self.addr_repo.get_by_address(req.from.clone()).await {
                Ok(Some(recv)) => recv,
                Ok(None) => return Err(Box::new(APIAddressError::AddressNotFound(req.from))),
                Err(e) => return Err(Box::new(APIAddressError::FindAddressError(e))),
            };

        if sender.balance < req.amount {
            return Err(Box::new(APITransactionError::BalanceNotEnough(
                req.from,
                sender.balance,
                req.amount,
            )));
        }

        let message = format!("{}{}{}", req.from, req.to, req.amount);

        let verify_result = crypto_helper::verify_signature(&message, &req.from, &req.signature);
        let is_valid = match verify_result {
            Ok(r) => r,
            Err(e) => {
                return Err(Box::new(APITransactionError::VerifySignatureError(
                    e.to_string(),
                )));
            }
        };

        if !is_valid {
            return Err(Box::new(APITransactionError::InvalidSignature));
        }

        let new_transaction = TransactionEntity::new(
            req.from.clone(),
            req.to.clone(),
            req.amount,
            req.signature.clone(),
            TransactionStatus::Pending,
            Arc::clone(&self.timer_helper),
        );

        return match self.tx_repo.insert(new_transaction).await {
            Ok(id) => Ok(id),
            Err(e) => Err(Box::new(APITransactionError::InsertTransactionError(e))),
        };
    }

}
