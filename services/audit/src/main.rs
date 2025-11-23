use axum::{
    routing::{get, post},
    Router,
};
use common::db;
use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;

use handlers::{AppState, create_audit_log, get_audit_logs};

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = env::var("AUDIT_DATABASE_URL").expect("AUDIT_DATABASE_URL must be set");

    let pool = db::create_pool(&db_url).await.expect("Failed to connect to Audit DB");

    let state = Arc::new(AppState {
        db: pool,
    });

    let app = Router::new()
        .route("/audit", post(create_audit_log))
        .route("/audit", get(get_audit_logs))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3003));
    tracing::info!("Audit Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
