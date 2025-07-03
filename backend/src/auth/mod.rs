use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::error::AppError;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

pub mod middleware;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64,  // user id
    pub exp: usize,  // expiration time
    pub role: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        let auth_header = if let Some(auth_header) = auth_header {
            auth_header
        } else {
            return Err(AppError::Auth("Missing authorization header".to_string()));
        };

        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            verify_token(token)
        } else {
            Err(AppError::Auth("Invalid authorization header format".to_string()))
        }
    }
}

pub fn create_token(user_id: i64, role: &str) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
        role: role.to_string(),
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let key = EncodingKey::from_secret(secret.as_bytes());

    encode(&Header::default(), &claims, &key)
        .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
}

pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());

    jsonwebtoken::decode::<Claims>(token, &key, &jsonwebtoken::Validation::default())
        .map(|data| data.claims)
        .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))
} 