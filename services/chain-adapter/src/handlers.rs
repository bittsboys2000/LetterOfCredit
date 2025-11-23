use axum::{
    extract::{State, Json},
    response::IntoResponse,
};
use common::{
    auth::AuthUser,
    error::AppError,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::ethereum::EthereumClient;

pub struct AppState {
    pub eth_client: EthereumClient,
}

#[derive(Deserialize)]
pub struct AnchorRequest {
    pub lc_id: String,
    pub state: String,
    pub hash: String, // Document hash
}

#[derive(Serialize)]
pub struct AnchorResponse {
    pub tx_hash: String,
}

pub async fn anchor_state(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<AnchorRequest>,
) -> Result<Json<AnchorResponse>, AppError> {
    // Allow internal services or authorized users
    // For now, just check if valid user
    if auth.user_id.is_nil() {
         return Err(AppError::Unauthorized("Invalid user".to_string()));
    }

    let tx_hash = state.eth_client.submit_hash(payload.lc_id, payload.state, payload.hash).await?;

    Ok(Json(AnchorResponse { tx_hash }))
}
