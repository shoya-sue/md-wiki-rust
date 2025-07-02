use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::error::AppError;

pub mod middleware;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,  // user id
    pub exp: usize,  // expiration time
    pub role: String,
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