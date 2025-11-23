use axum::{
    routing::post,
    Router,
};
use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ethereum;
mod handlers;

use handlers::{AppState, anchor_state};
use ethereum::EthereumClient;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rpc_url = env::var("BESU_RPC_URL").expect("BESU_RPC_URL must be set");
    let contract_addr = env::var("CONTRACT_ADDRESS").expect("CONTRACT_ADDRESS must be set");

    let eth_client = EthereumClient::new(rpc_url, contract_addr);

    let state = Arc::new(AppState {
        eth_client,
    });

    let app = Router::new()
        .route("/anchor", post(anchor_state))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3004));
    tracing::info!("Chain Adapter Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
