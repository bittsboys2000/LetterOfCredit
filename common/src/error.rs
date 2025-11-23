use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("IPFS error: {0}")]
    Ipfs(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Ipfs(e) => (StatusCode::BAD_GATEWAY, e),
            AppError::Encryption(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::NotFound(e) => (StatusCode::NOT_FOUND, e),
            AppError::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e),
            AppError::BadRequest(e) => (StatusCode::BAD_REQUEST, e),
            AppError::Internal(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
