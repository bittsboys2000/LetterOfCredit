use axum::{
    extract::{Multipart, Path, State},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use common::{
    auth::AuthUser,
    error::AppError,
    models::{DocumentMetadata, UploadResponse},
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use sha2::{Sha256, Digest};
use crate::{encryption, ipfs::IpfsClient};

pub struct AppState {
    pub db: PgPool,
    pub ipfs: IpfsClient,
}

pub async fn upload_document(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;

        // 1. Hash File
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let file_hash = hex::encode(hasher.finalize());

        // 2. Encrypt
        let (encrypted_data, nonce) = encryption::encrypt(&data)?;

        // 3. Upload to IPFS
        let ipfs_cid = state.ipfs.add(encrypted_data).await?;

        // 4. Store Metadata
        let doc_id = Uuid::new_v4();
        let metadata = DocumentMetadata {
            id: doc_id,
            owner_id: auth.user_id,
            filename,
            content_type,
            ipfs_cid: ipfs_cid.clone(),
            file_hash: file_hash.clone(),
            encryption_iv: hex::encode(nonce),
            created_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO documents (id, owner_id, filename, content_type, ipfs_cid, file_hash, encryption_iv, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(metadata.id)
        .bind(metadata.owner_id)
        .bind(metadata.filename)
        .bind(metadata.content_type)
        .bind(metadata.ipfs_cid)
        .bind(metadata.file_hash)
        .bind(metadata.encryption_iv)
        .bind(metadata.created_at)
        .execute(&state.db)
        .await?;

        return Ok(Json(UploadResponse {
            document_id: doc_id,
            ipfs_cid,
            file_hash,
        }));
    }

    Err(AppError::BadRequest("No file provided".to_string()))
}

pub async fn get_document(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let doc = sqlx::query_as::<_, DocumentMetadata>(
        "SELECT * FROM documents WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("Document not found".to_string()))?;

    // Check access (simple check: owner or auditor)
    if doc.owner_id != auth.user_id && auth.role != "Auditor" {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    // 1. Fetch from IPFS
    let encrypted_data = state.ipfs.cat(&doc.ipfs_cid).await?;

    // 2. Decrypt
    let nonce = hex::decode(&doc.encryption_iv).map_err(|_| AppError::Internal("Invalid IV".to_string()))?;
    let decrypted_data = encryption::decrypt(&encrypted_data, &nonce)?;

    // 3. Return
    Ok((
        [(axum::http::header::CONTENT_TYPE, doc.content_type)],
        decrypted_data,
    ))
}
