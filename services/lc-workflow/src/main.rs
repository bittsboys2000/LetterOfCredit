use axum::{
    routing::{get, post, put},
    Router,
};
use common::db;
use dotenvy::dotenv;
use std::{env, net::SocketAddr, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod models;

use handlers::{AppState, create_lc, get_lc, update_lc_status};

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

    let pool = db::create_pool(&db_url).await.expect("Failed to connect to DB");

    let state = Arc::new(AppState {
        db: pool,
    });

    let app = Router::new()
        .route("/lc", post(create_lc))
        .route("/lc/:id", get(get_lc))
        .route("/lc/:id/status", put(update_lc_status))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3002));
    tracing::info!("LC Workflow Service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
