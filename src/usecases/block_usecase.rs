use std::sync::Arc;

use crate::{
    entities::block_entity::BlockEntity,
    errors::{block_error::APIBlockError, error::IntoErrorResponse},
    models::address_model::CoinWithAddress,
    repository::block_repository::SharedBlockRepository,
    timer_helper::IntoTimerHelperShared,
    usecases::transaction_usecase::TransactionUsecase,
};
use bson::oid::ObjectId;
use sha2::{Digest, Sha256};

use super::address_usecase::AddressUsecase;

pub struct BlockUsecase {
    block_repo: SharedBlockRepository,
    tx_usecase: Arc<TransactionUsecase>,
    addr_usecase: Arc<AddressUsecase>,
    timer_helper: IntoTimerHelperShared,
}

impl BlockUsecase {
    pub fn creation(
        block_repo: SharedBlockRepository,
        tx_usecase: Arc<TransactionUsecase>,
        addr_usecase: Arc<AddressUsecase>,
        timer_helper: IntoTimerHelperShared,
    ) -> Arc<Self> {
        Arc::new(Self {
            block_repo,
            tx_usecase,
            addr_usecase,
            timer_helper,
        })
    }

    pub async fn build_block(&self) -> Result<ObjectId, Box<dyn IntoErrorResponse>> {
        let latest_block = self.block_repo.find_latest().await.map_err(|e| {
            Box::new(APIBlockError::FindBlockError(e)) as Box<dyn IntoErrorResponse>
        })?;

        let previous_hash = latest_block.clone().map(|b| b.hash).unwrap_or_default();
        let index = latest_block.map(|b| b.index + 1).unwrap_or(1);

        let txs = self.tx_usecase.get_all_pending().await?;

        let tx_ids: Vec<ObjectId> = txs.iter().filter_map(|tx| tx.id).collect();

        let raw = format!("{:?}{:?}", tx_ids, previous_hash);
        let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));

        let block = BlockEntity::new(
            index,
            tx_ids.clone(),
            previous_hash,
            hash.clone(),
            0,
            Arc::clone(&self.timer_helper),
        );

        let inserted_id = match self.block_repo.insert(block).await {
            Ok(id) => id,
            Err(e) => return Err(Box::new(APIBlockError::InsertBlockError(e))),
        };

        for tx_id in tx_ids {
            let tx = self.tx_usecase.get_by_id(tx_id).await?;

            if self
                .addr_usecase
                .withdraw_coin(CoinWithAddress {
                    public_key: tx.from.clone(),
                    amount: tx.amount,
                })
                .await
                .is_err()
            {
                self.tx_usecase.reject_transaction(tx_id).await.ok();
                continue;
            }

            if self
                .addr_usecase
                .deposit_coin(CoinWithAddress {
                    public_key: tx.to.clone(),
                    amount: tx.amount,
                })
                .await
                .is_err()
            {
                self.addr_usecase
                    .deposit_coin(CoinWithAddress {
                        public_key: tx.from,
                        amount: tx.amount,
                    })
                    .await
                    .ok(); // rollback withdraw
                self.tx_usecase.reject_transaction(tx_id).await.ok();
                continue;
            }

            if self
                .tx_usecase
                .confirm_transaction(tx_id, hash.clone())
                .await
                .is_err()
            {
                self.tx_usecase.reject_transaction(tx_id).await.ok();
                self.addr_usecase
                    .withdraw_coin(CoinWithAddress {
                        public_key: tx.to,
                        amount: tx.amount,
                    })
                    .await
                    .ok(); // rollback deposit
                self.addr_usecase
                    .deposit_coin(CoinWithAddress {
                        public_key: tx.from,
                        amount: tx.amount,
                    })
                    .await
                    .ok(); // rollback withdraw
                continue;
            }
        }

        Ok(inserted_id)
    }

    pub async fn get_block_by_hash(
        &self,
        hash: String,
    ) -> Result<BlockEntity, Box<dyn IntoErrorResponse>> {
        return match self.block_repo.find_by_hash(hash.clone()).await {
            Ok(Some(block)) => Ok(block),
            Ok(None) => return Err(Box::new(APIBlockError::NotFound(hash))),
            Err(e) => return Err(Box::new(APIBlockError::FindBlockError(e))),
        };
    }

    pub async fn get_latest_block(&self) -> Result<BlockEntity, Box<dyn IntoErrorResponse>> {
        return match self.block_repo.find_latest().await {
            Ok(Some(block)) => Ok(block),
            Ok(None) => {
                return Err(Box::new(APIBlockError::NotFound(String::from(
                    "block latest",
                ))));
            }
            Err(e) => return Err(Box::new(APIBlockError::FindBlockError(e))),
        };
    }

    pub async fn verify_chain(&self) -> Result<(), Box<dyn IntoErrorResponse>> {
        return match self.block_repo.is_chain_valid().await {
            Ok(()) => Ok(()),
            Err(e) => return Err(Box::new(APIBlockError::InvalidChain(e))),
        };
    }
}
