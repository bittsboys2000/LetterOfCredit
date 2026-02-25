use axum::{
    routing::{get, post},
    Router,
};
use common::db;
use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod encryption;
mod handlers;
mod ipfs;

use handlers::{AppState, get_document, upload_document};
use ipfs::IpfsClient;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let ipfs_url = env::var("IPFS_API_URL").expect("IPFS_API_URL must be set");

    let pool = db::create_pool(&db_url).await.expect("Failed to connect to DB");
    let ipfs_client = IpfsClient::new(ipfs_url);

    let state = Arc::new(AppState {
        db: pool,
        ipfs: ipfs_client,
    });

    let app = Router::new()
        .route("/documents", post(upload_document))
        .route("/documents/:id", get(get_document))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("Document Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
