use crate::models::address_model::{CoinWithAddress, InsertAddress};
use crate::usecases::address_usecase::AddressUsecase;
use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

fn generate_keypair() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    return (secret_key, public_key);
}

pub async fn handler_create_address(address_usecase: Arc<AddressUsecase>) -> impl IntoResponse {
    let (secret_key, public_key) = generate_keypair();
    let object_id = match address_usecase
        .create_new_address(InsertAddress {
            public_key: public_key.to_string(),
        })
        .await
    {
        Ok(id) => id,
        Err(e) => return e.error().into_response(),
    };

    let secret_key_hex = hex::encode(secret_key.secret_bytes());

    return (
        StatusCode::CREATED,
        Json(json!({
            "object_id": object_id,
            "public_key": public_key.to_string(),
            "secret_key": secret_key_hex,
        })),
    )
        .into_response();
}

#[derive(Deserialize)]
pub struct DepositRequest {
    pub amount: u64,
}

pub async fn handler_deposit_coin(
    Path(public_key): Path<String>,
    Json(payload): Json<DepositRequest>,
    address_usecase: Arc<AddressUsecase>,
) -> impl IntoResponse {
    let deposit_info = CoinWithAddress {
        public_key,
        amount: payload.amount,
    };
    match address_usecase.deposit_coin(deposit_info).await {
        Ok(()) => (),
        Err(e) => return e.error().into_response(),
    };

    return (
        StatusCode::OK,
        Json(json!({
            "success": true,
        })),
    )
        .into_response();
}
