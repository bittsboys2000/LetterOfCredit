use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::types::Json;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub event_type: String,
    pub actor_id: Uuid,
    pub resource_id: Option<Uuid>,
    pub details: Json<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuditLogRequest {
    pub event_type: String,
    pub actor_id: Uuid,
    pub resource_id: Option<Uuid>,
    pub details: serde_json::Value,
}
