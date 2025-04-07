use std::sync::Arc;

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::usecases::block_usecase::BlockUsecase;

pub async fn handler_build_block(block_usecase: Arc<BlockUsecase>) -> impl IntoResponse {
    let result = match block_usecase.build_block().await {
        Ok(id) => json!({ "success": true, "block_id": id.to_string() }),
        Err(e) => return e.error().into_response(),
    };

    (StatusCode::OK, Json(result)).into_response()
}

pub async fn handler_get_latest_block(block_usecase: Arc<BlockUsecase>) -> impl IntoResponse {
    let result = match block_usecase.get_latest_block().await {
        Ok(block) => json!({ "success": true, "block": block }),
        Err(e) => return e.error().into_response(),
    };

    (StatusCode::OK, Json(result)).into_response()
}

pub async fn handler_get_block_by_hash(
    Path(hash): Path<String>,
    block_usecase: Arc<BlockUsecase>,
) -> impl IntoResponse {
    let result = match block_usecase.get_block_by_hash(hash).await {
        Ok(block) => json!({ "success": true, "block": block }),
        Err(e) => return e.error().into_response(),
    };

    (StatusCode::OK, Json(result)).into_response()
}

pub async fn handler_verify_chain(block_usecase: Arc<BlockUsecase>) -> impl IntoResponse {
    let result = match block_usecase.verify_chain().await {
        Ok(_) => json!({ "success": true }),
        Err(e) => return e.error().into_response(),
    };

    (StatusCode::OK, Json(result)).into_response()
}
