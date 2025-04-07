use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    http::Method,
    routing::{get, patch, post},
};
use rust_chain::{
    database::database,
    handlers::{
        address_handler::{handler_create_address, handler_deposit_coin},
        block_handler::{
            handler_build_block, handler_get_block_by_hash, handler_get_latest_block,
            handler_verify_chain,
        },
        transaction_handler::{
            handler_confirm_transaction, handler_create_transaction,
            handler_get_pending_transactions, handler_get_transaction_by_id,
            handler_get_transactions_by_address,
        },
    },
    repository::{
        address_repository::MongoAddressRepository, block_repository::MongoBlockRepository,
        transaction_repository::MongoTransactionRepository,
    },
    setting::Setting,
    timer_helper::TimerHelper,
    usecases::{
        address_usecase::AddressUsecase, block_usecase::BlockUsecase,
        transaction_usecase::TransactionUsecase,
    },
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let setting = Setting::new().unwrap();
    info!("Setting has been loaded.");

    let db = database::db_connect(Arc::clone(&setting)).await.unwrap();
    info!("database connect successfully");

    let timer_helper = TimerHelper::Directly.creation();

    let address_repository = MongoAddressRepository::creation(db.clone());
    let address_usecase =
        AddressUsecase::creation(Arc::clone(&address_repository), Arc::clone(&timer_helper));

    let transaction_repository = MongoTransactionRepository::creation(db.clone());
    let transaction_usecase = TransactionUsecase::creation(
        Arc::clone(&transaction_repository),
        Arc::clone(&address_repository),
        Arc::clone(&timer_helper),
    );

    let block_repository = MongoBlockRepository::creation(db.clone());
    let block_usecase = BlockUsecase::creation(
        Arc::clone(&block_repository),
        Arc::clone(&transaction_usecase),
        Arc::clone(&address_usecase),
        Arc::clone(&timer_helper),
    );

    let app = Router::new()
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_origin(Any),
        )
        .layer(TraceLayer::new_for_http())
        .merge(address_routes(Arc::clone(&address_usecase)))
        .merge(transaction_routes(Arc::clone(&transaction_usecase)))
        .merge(block_routes(Arc::clone(&block_usecase)));

    let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], setting.server.port as u16));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("server start at {:?}", addr);

    axum::serve(listener, app).await.unwrap();
}

fn address_routes(address_usecase: Arc<AddressUsecase>) -> Router {
    return Router::<()>::new()
        .route(
            "/addresses",
            post({
                let usecase = Arc::clone(&address_usecase);
                move || handler_create_address(usecase)
            }),
        )
        .route(
            "/addresses/{public_key}",
            patch({
                let usecase = Arc::clone(&address_usecase);
                move |path, body| handler_deposit_coin(path, body, usecase)
            }),
        );
}

fn transaction_routes(transaction_usecase: Arc<TransactionUsecase>) -> Router {
    return Router::<()>::new()
        .route(
            "/transactions",
            post({
                let usecase = Arc::clone(&transaction_usecase);
                move |body| handler_create_transaction(body, usecase)
            }),
        )
        .route(
            "/transactions/{id}",
            get({
                let usecase = Arc::clone(&transaction_usecase);
                move |path| handler_get_transaction_by_id(path, usecase)
            }),
        )
        .route(
            "/addresses/{address}/transactions",
            get({
                let usecase = Arc::clone(&transaction_usecase);
                move |path| handler_get_transactions_by_address(path, usecase)
            }),
        )
        .route(
            "/transactions/pending",
            get({
                let usecase = Arc::clone(&transaction_usecase);
                move || handler_get_pending_transactions(usecase)
            }),
        )
        .route(
            "/transactions/{id}/confirm",
            patch({
                let usecase = Arc::clone(&transaction_usecase);
                move |path, body| handler_confirm_transaction(path, body, usecase)
            }),
        );
}

fn block_routes(block_usecase: Arc<BlockUsecase>) -> Router {
    return Router::<()>::new()
        .route(
            "/blocks",
            post({
                let usecase = Arc::clone(&block_usecase);
                move || handler_build_block(usecase)
            }),
        )
        .route(
            "/blocks/latest",
            get({
                let usecase = Arc::clone(&block_usecase);
                move || handler_get_latest_block(usecase)
            }),
        )
        .route(
            "/blocks/{hash}",
            get({
                let usecase = Arc::clone(&block_usecase);
                move |path| handler_get_block_by_hash(path, usecase)
            }),
        )
        .route(
            "/blocks/verify",
            get({
                let usecase = Arc::clone(&block_usecase);
                move || handler_verify_chain(usecase)
            }),
        );
}
