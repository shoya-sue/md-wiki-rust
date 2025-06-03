use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::{error::AppError, models::Role};
use std::time::{SystemTime, UNIX_EPOCH};

const JWT_SECRET: &[u8] = b"your-secret-key";  // 本番環境では環境変数から取得すべき

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,  // user_id
    pub username: String,
    pub role: Role,
    pub exp: i64,  // expiration time
}

impl Claims {
    pub fn new(user_id: i64, username: String, role: Role) -> Self {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64 + 24 * 60 * 60; // 24時間後に期限切れ

        Self {
            sub: user_id,
            username,
            role,
            exp,
        }
    }
}

pub fn create_token(claims: &Claims) -> Result<String, AppError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
    .map_err(|e| AppError::Auth(format!("Failed to create token: {}", e)))
}

pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))
} 