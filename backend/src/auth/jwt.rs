use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::User;

// JWTクレーム
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    // サブジェクト（ユーザーID）
    pub sub: String,
    // ユーザー名
    pub username: String,
    // ユーザーロール
    pub role: String,
    // 発行時間
    pub iat: u64,
    // 有効期限
    pub exp: u64,
}

// JWTの秘密鍵
const JWT_SECRET: &[u8] = b"md_wiki_secret_key"; // 本番環境では環境変数などから取得するべき

// トークンの有効期限（24時間）
const TOKEN_EXPIRATION: u64 = 60 * 60 * 24;

// ユーザー情報からJWTトークンを生成
pub fn generate_token(user: &User) -> Result<String, JwtError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.to_string(),
        iat: now,
        exp: now + TOKEN_EXPIRATION,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}

// JWTトークンを検証して、クレームを取得
pub fn verify_token(token: &str) -> Result<Claims, JwtError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )?;
    
    Ok(token_data.claims)
}

// 現在のUnixタイムスタンプを取得
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
} 