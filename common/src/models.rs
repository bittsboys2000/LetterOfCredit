use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DocumentMetadata {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub ipfs_cid: String,
    pub file_hash: String, // SHA-256
    pub encryption_iv: String, // Hex encoded
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub document_id: Uuid,
    pub ipfs_cid: String,
    pub file_hash: String,
}
