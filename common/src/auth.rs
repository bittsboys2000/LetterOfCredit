use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // User ID
    pub role: String,
    pub exp: usize,
}

pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer MOCK_TOKEN_") => {
                let role = &header[18..];
                let user_id = match role {
                    "Importer" => "00000000-0000-0000-0000-000000000001",
                    "Exporter" => "00000000-0000-0000-0000-000000000002",
                    "IssuingBank" => "00000000-0000-0000-0000-000000000003",
                    "AdvisingBank" => "00000000-0000-0000-0000-000000000004",
                    "Auditor" => "00000000-0000-0000-0000-000000000005",
                    _ => "00000000-0000-0000-0000-000000000001",
                };
                Ok(AuthUser {
                    user_id: Uuid::parse_str(user_id).unwrap(),
                    role: role.to_string(),
                })
            }
            Some(header) if header.starts_with("Bearer ") => {
                let token = &header[7..];
                let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretkey".to_string());
                
                let token_data = decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(secret.as_bytes()),
                    &Validation::default(),
                )
                .map_err(|_| StatusCode::UNAUTHORIZED)?;

                Ok(AuthUser {
                    user_id: token_data.claims.sub,
                    role: token_data.claims.role,
                })
            }
            _ => Err(StatusCode::UNAUTHORIZED),
        }
    }
}
