use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::Type;

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Clone)]
#[sqlx(type_name = "lc_status", rename_all = "lowercase")]
pub enum LcStatus {
    Draft,
    Submitted,
    Review,
    Approved,
    Rejected,
    Closed,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LetterOfCredit {
    pub id: Uuid,
    pub applicant_id: Uuid, // Importer
    pub beneficiary_id: Uuid, // Exporter
    pub issuing_bank_id: Uuid,
    pub advising_bank_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub expiry_date: DateTime<Utc>,
    pub status: LcStatus,
    pub document_ids: Vec<Uuid>, // Array of document IDs
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLcRequest {
    pub beneficiary_id: Uuid,
    pub issuing_bank_id: Uuid,
    pub advising_bank_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub expiry_date: DateTime<Utc>,
    pub document_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: LcStatus,
}
