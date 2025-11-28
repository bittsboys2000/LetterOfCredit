use axum::{
    extract::{Path, State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use common::{
    auth::AuthUser,
    error::AppError,
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{LetterOfCredit, LcStatus, CreateLcRequest, UpdateStatusRequest, ListLcQuery, LcListResponse};

pub struct AppState {
    pub db: PgPool,
    pub chain_adapter_url: Option<String>,
    pub audit_service_url: Option<String>,
}

pub async fn create_lc(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateLcRequest>,
) -> Result<Json<LetterOfCredit>, AppError> {
    // Role check: Only Importer can create LC
    if auth.role != "Importer" {
        return Err(AppError::Unauthorized("Only Importers can create LCs".to_string()));
    }

    let lc = LetterOfCredit {
        id: Uuid::new_v4(),
        applicant_id: auth.user_id,
        beneficiary_id: payload.beneficiary_id,
        issuing_bank_id: payload.issuing_bank_id,
        advising_bank_id: payload.advising_bank_id,
        amount: payload.amount,
        currency: payload.currency,
        expiry_date: payload.expiry_date,
        status: LcStatus::Draft,
        document_ids: payload.document_ids,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    sqlx::query(
        r#"
        INSERT INTO letter_of_credits (id, applicant_id, beneficiary_id, issuing_bank_id, advising_bank_id, amount, currency, expiry_date, status, document_ids, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#
    )
    .bind(lc.id)
    .bind(lc.applicant_id)
    .bind(lc.beneficiary_id)
    .bind(lc.issuing_bank_id)
    .bind(lc.advising_bank_id)
    .bind(lc.amount)
    .bind(lc.currency.clone())
    .bind(lc.expiry_date)
    .bind(lc.status.clone() as LcStatus)
    .bind(&lc.document_ids)
    .bind(lc.created_at)
    .bind(lc.updated_at)
    .execute(&state.db)
    .await?;

    Ok(Json(lc))
}

pub async fn get_lc(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<LetterOfCredit>, AppError> {
    let lc = sqlx::query_as::<_, LetterOfCredit>(
        r#"SELECT id, applicant_id, beneficiary_id, issuing_bank_id, advising_bank_id, amount, currency, expiry_date, status, document_ids, created_at, updated_at FROM letter_of_credits WHERE id = $1"#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("LC not found".to_string()))?;

    // Access control (simplified)
    if lc.applicant_id != auth.user_id && lc.beneficiary_id != auth.user_id && lc.issuing_bank_id != auth.user_id && lc.advising_bank_id != auth.user_id && auth.role != "Auditor" {
         return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    Ok(Json(lc))
}

pub async fn list_lcs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<ListLcQuery>,
) -> Result<Json<LcListResponse>, AppError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    // Build query based on role - users can only see LCs they're involved in (unless Auditor)
    let lcs = if auth.role == "Auditor" {
        // Auditors can see all LCs
        sqlx::query_as::<_, LetterOfCredit>(
            r#"SELECT id, applicant_id, beneficiary_id, issuing_bank_id, advising_bank_id, amount, currency, expiry_date, status, document_ids, created_at, updated_at 
               FROM letter_of_credits 
               ORDER BY updated_at DESC
               LIMIT $1 OFFSET $2"#
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await?
    } else {
        // Other users see LCs where they're a participant
        sqlx::query_as::<_, LetterOfCredit>(
            r#"SELECT id, applicant_id, beneficiary_id, issuing_bank_id, advising_bank_id, amount, currency, expiry_date, status, document_ids, created_at, updated_at 
               FROM letter_of_credits 
               WHERE applicant_id = $1 OR beneficiary_id = $1 OR issuing_bank_id = $1 OR advising_bank_id = $1
               ORDER BY updated_at DESC
               LIMIT $2 OFFSET $3"#
        )
        .bind(auth.user_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await?
    };

    // Get total count for pagination
    let total: (i64,) = if auth.role == "Auditor" {
        sqlx::query_as("SELECT COUNT(*) FROM letter_of_credits")
            .fetch_one(&state.db)
            .await?
    } else {
        sqlx::query_as(
            "SELECT COUNT(*) FROM letter_of_credits WHERE applicant_id = $1 OR beneficiary_id = $1 OR issuing_bank_id = $1 OR advising_bank_id = $1"
        )
        .bind(auth.user_id)
        .fetch_one(&state.db)
        .await?
    };

    Ok(Json(LcListResponse {
        items: lcs,
        total: total.0 as u64,
        limit,
        offset,
    }))
}

pub async fn update_lc_status(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<LetterOfCredit>, AppError> {
    let lc = sqlx::query_as::<_, LetterOfCredit>(
        r#"SELECT id, applicant_id, beneficiary_id, issuing_bank_id, advising_bank_id, amount, currency, expiry_date, status, document_ids, created_at, updated_at FROM letter_of_credits WHERE id = $1"#
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("LC not found".to_string()))?;

    let old_status = lc.status.clone();

    // State Machine Logic
    match (lc.status.clone(), payload.status.clone()) {
        (LcStatus::Draft, LcStatus::Submitted) => {
            if auth.user_id != lc.applicant_id { return Err(AppError::Unauthorized("Only Applicant can submit".to_string())); }
        },
        (LcStatus::Submitted, LcStatus::Review) => {
            if auth.user_id != lc.issuing_bank_id { return Err(AppError::Unauthorized("Only Issuing Bank can start review".to_string())); }
        },
        (LcStatus::Review, LcStatus::Approved) => {
            if auth.user_id != lc.issuing_bank_id { return Err(AppError::Unauthorized("Only Issuing Bank can approve".to_string())); }
        },
        (LcStatus::Review, LcStatus::Rejected) => {
             if auth.user_id != lc.issuing_bank_id { return Err(AppError::Unauthorized("Only Issuing Bank can reject".to_string())); }
        },
        (LcStatus::Approved, LcStatus::Closed) => {
             if auth.user_id != lc.issuing_bank_id { return Err(AppError::Unauthorized("Only Issuing Bank can close".to_string())); }
        },
        _ => return Err(AppError::BadRequest(format!("Invalid transition from {:?} to {:?}", lc.status, payload.status))),
    }

    let updated_lc = sqlx::query_as::<_, LetterOfCredit>(
        r#"
        UPDATE letter_of_credits
        SET status = $1, updated_at = $2
        WHERE id = $3
        RETURNING id, applicant_id, beneficiary_id, issuing_bank_id, advising_bank_id, amount, currency, expiry_date, status, document_ids, created_at, updated_at
        "#
    )
    .bind(payload.status.clone() as LcStatus)
    .bind(Utc::now())
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    // Anchor state change to blockchain (fire and forget for now)
    if let Some(chain_url) = &state.chain_adapter_url {
        let chain_url = chain_url.clone();
        let lc_id = id.to_string();
        let new_status = format!("{:?}", payload.status);
        let doc_hash = lc.document_ids.first().map(|d| d.to_string()).unwrap_or_default();
        let auth_token = format!("MOCK_TOKEN_{}", auth.role);
        
        tokio::spawn(async move {
            if let Err(e) = anchor_to_chain(&chain_url, &lc_id, &new_status, &doc_hash, &auth_token).await {
                tracing::error!("Failed to anchor to chain: {}", e);
            }
        });
    }

    // Log to audit service
    if let Some(audit_url) = &state.audit_service_url {
        let audit_url = audit_url.clone();
        let event = AuditEvent {
            event_type: "LC_STATUS_UPDATE".to_string(),
            actor_id: auth.user_id,
            resource_id: Some(id),
            details: serde_json::json!({
                "old_status": format!("{:?}", old_status),
                "new_status": format!("{:?}", payload.status),
                "lc_id": id.to_string(),
            }),
        };
        
        tokio::spawn(async move {
            if let Err(e) = log_audit_event(&audit_url, event).await {
                tracing::error!("Failed to log audit event: {}", e);
            }
        });
    }

    Ok(Json(updated_lc))
}

#[derive(serde::Serialize)]
struct ChainAnchorRequest {
    lc_id: String,
    state: String,
    hash: String,
}

async fn anchor_to_chain(base_url: &str, lc_id: &str, state: &str, hash: &str, auth_token: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/anchor", base_url))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&ChainAnchorRequest {
            lc_id: lc_id.to_string(),
            state: state.to_string(),
            hash: hash.to_string(),
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        tracing::info!("Successfully anchored LC {} state {} to blockchain", lc_id, state);
        Ok(())
    } else {
        Err(format!("Chain adapter returned status: {}", res.status()))
    }
}

#[derive(serde::Serialize)]
struct AuditEvent {
    event_type: String,
    actor_id: Uuid,
    resource_id: Option<Uuid>,
    details: serde_json::Value,
}

async fn log_audit_event(base_url: &str, event: AuditEvent) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/audit", base_url))
        .json(&event)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        tracing::info!("Successfully logged audit event: {}", event.event_type);
        Ok(())
    } else {
        Err(format!("Audit service returned status: {}", res.status()))
    }
}
