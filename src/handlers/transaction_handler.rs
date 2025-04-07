use std::sync::Arc;

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use bson::oid::ObjectId;
use serde::Deserialize;
use serde_json::json;

use crate::{
    models::transaction_model::CreateTransactionRequest,
    usecases::transaction_usecase::TransactionUsecase,
};

pub async fn handler_create_transaction(
    Json(payload): Json<CreateTransactionRequest>,
    tx_usecase: Arc<TransactionUsecase>,
) -> impl IntoResponse {
    let object_id = match tx_usecase.create_transaction(payload).await {
        Ok(r) => r,
        Err(e) => return e.error().into_response(),
    };

    return (
        StatusCode::CREATED,
        Json(json!({
            "object_id": object_id
        })),
    )
        .into_response();
}

pub async fn handler_get_transaction_by_id(
    Path(tx_id): Path<ObjectId>,
    tx_usecase: Arc<TransactionUsecase>,
) -> impl IntoResponse {
    let tx = match tx_usecase.get_by_id(tx_id).await {
        Ok(tx) => tx,
        Err(e) => return e.error().into_response(),
    };

    return (
        StatusCode::OK,
        Json(json!({
            "tx": tx,
        })),
    )
        .into_response();
}

pub async fn handler_get_transactions_by_address(
    Path(address): Path<String>,
    tx_usecase: Arc<TransactionUsecase>,
) -> impl IntoResponse {
    let txs = match tx_usecase.get_by_address(address).await {
        Ok(txs) => txs,
        Err(e) => return e.error().into_response(),
    };

    return (
        StatusCode::OK,
        Json(json!({
            "txs": txs,
        })),
    )
        .into_response();
}

pub async fn handler_get_pending_transactions(
    tx_usecase: Arc<TransactionUsecase>,
) -> impl IntoResponse {
    let txs = match tx_usecase.get_all_pending().await {
        Ok(txs) => txs,
        Err(e) => return e.error().into_response(),
    };

    return (
        StatusCode::OK,
        Json(json!({
            "txs": txs,
        })),
    )
        .into_response();
}

#[derive(Deserialize)]
pub struct ConfirmTxRequest {
    pub block_hash: String,
}

pub async fn handler_confirm_transaction(
    Path(tx_id): Path<ObjectId>,
    Json(payload): Json<ConfirmTxRequest>,
    tx_usecase: Arc<TransactionUsecase>,
) -> impl IntoResponse {
    return match tx_usecase
        .confirm_transaction(tx_id, payload.block_hash)
        .await
    {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(e) => e.error().into_response(),
    };
}

