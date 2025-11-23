use axum::{
    extract::{Path, State},
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
use crate::models::{LetterOfCredit, LcStatus, CreateLcRequest, UpdateStatusRequest};

pub struct AppState {
    pub db: PgPool,
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
             // Maybe beneficiary or banks can close?
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
    .bind(payload.status as LcStatus)
    .bind(Utc::now())
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    // TODO: Emit event to Chain Adapter here

    Ok(Json(updated_lc))
}
