use axum::{
    extract::{Path, State, Query},
    Json,
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
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, Header, EncodingKey};
use crate::models::{User, UserResponse, RegisterRequest, LoginRequest, AuthResponse, JwtClaims};

pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
}

const TOKEN_EXPIRY_HOURS: u64 = 24;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Validate email format
    if !payload.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    // Validate password strength
    if payload.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".to_string()));
    }

    // Check if email already exists
    let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(&state.db)
        .await?;

    if existing > 0 {
        return Err(AppError::BadRequest("Email already registered".to_string()));
    }

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?
        .to_string();

    // Create user
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, password_hash, name, organization, role, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, true, $7, $7)
        RETURNING id, email, password_hash, name, organization, role, is_active, created_at, updated_at
        "#
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.name)
    .bind(&payload.organization)
    .bind(&payload.role)
    .bind(now)
    .fetch_one(&state.db)
    .await?;

    // Generate JWT
    let token = generate_token(&user, &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_EXPIRY_HOURS * 3600,
        user: user.into(),
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Find user by email
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, name, organization, role, is_active, created_at, updated_at FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("Account is disabled".to_string()));
    }

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {}", e)))?;
    
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Invalid email or password".to_string()))?;

    // Generate JWT
    let token = generate_token(&user, &state.jwt_secret)?;

    // Update last login (fire and forget)
    let _ = sqlx::query("UPDATE users SET updated_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(user.id)
        .execute(&state.db)
        .await;

    Ok(Json(AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_EXPIRY_HOURS * 3600,
        user: user.into(),
    }))
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, name, organization, role, is_active, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

#[derive(serde::Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    // Only admins and auditors can list all users
    if auth.role != "Admin" && auth.role != "Auditor" {
        return Err(AppError::Unauthorized("Only admins can list users".to_string()));
    }

    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let users = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, name, organization, role, is_active, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(users.into_iter().map(|u| u.into()).collect()))
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    // Users can view their own profile, admins/auditors can view anyone
    if auth.user_id != id && auth.role != "Admin" && auth.role != "Auditor" {
        return Err(AppError::Unauthorized("Access denied".to_string()));
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, name, organization, role, is_active, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

fn generate_token(user: &User, secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = now + chrono::Duration::hours(TOKEN_EXPIRY_HOURS as i64);
    
    let claims = JwtClaims {
        sub: user.id,
        email: user.email.clone(),
        role: user.role.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))
}

