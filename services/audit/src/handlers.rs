use axum::{
    extract::{State, Query},
    Json,
    response::IntoResponse,
};
use common::{
    auth::AuthUser,
    error::AppError,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{AuditLog, CreateAuditLogRequest};

pub struct AppState {
    pub db: PgPool,
}

pub async fn create_audit_log(
    State(state): State<Arc<AppState>>,
    // Internal service, might not have user auth context in the same way, 
    // but let's assume it's called by other services or authenticated users.
    // For simplicity, we'll allow it to be called publicly for now (or protected by internal API key in real world).
    Json(payload): Json<CreateAuditLogRequest>,
) -> Result<Json<AuditLog>, AppError> {
    let log = AuditLog {
        id: Uuid::new_v4(),
        event_type: payload.event_type,
        actor_id: payload.actor_id,
        resource_id: payload.resource_id,
        details: sqlx::types::Json(payload.details),
        timestamp: Utc::now(),
    };

    sqlx::query(
        r#"
        INSERT INTO audit_logs (id, event_type, actor_id, resource_id, details, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#
    )
    .bind(log.id)
    .bind(log.event_type.clone())
    .bind(log.actor_id)
    .bind(log.resource_id)
    .bind(&log.details)
    .bind(log.timestamp)
    .execute(&state.db)
    .await?;

    Ok(Json(log))
}

#[derive(serde::Deserialize)]
pub struct AuditQuery {
    pub actor_id: Option<Uuid>,
}

pub async fn get_audit_logs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLog>>, AppError> {
    if auth.role != "Auditor" {
        return Err(AppError::Unauthorized("Only Auditors can view logs".to_string()));
    }

    let logs = if let Some(actor_id) = params.actor_id {
        sqlx::query_as::<_, AuditLog>(
            r#"SELECT id, event_type, actor_id, resource_id, details, timestamp FROM audit_logs WHERE actor_id = $1 ORDER BY timestamp DESC"#
        )
        .bind(actor_id)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, AuditLog>(
            r#"SELECT id, event_type, actor_id, resource_id, details, timestamp FROM audit_logs ORDER BY timestamp DESC LIMIT 100"#
        )
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(logs))
}
