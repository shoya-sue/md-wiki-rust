use serde::{Deserialize, Serialize};
use argon2::{self, Config, ThreadMode, Variant, Version};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: i64,
    pub last_login: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

impl UserRole {
    pub fn from_str(role: &str) -> Self {
        match role {
            "admin" => UserRole::Admin,
            "editor" => UserRole::Editor,
            _ => UserRole::Viewer,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::Editor => "editor".to_string(),
            UserRole::Viewer => "viewer".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistration {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub fn hash_password(password: &str) -> Result<String, argon2::Error> {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 65536, // 64MB
        time_cost: 3,    // 3 iterations
        lanes: 4,        // 4 parallel lanes
        thread_mode: ThreadMode::Parallel,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    argon2::hash_encoded(password.as_bytes(), &salt, &config)
}

pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::Error> {
    argon2::verify_encoded(hash, password.as_bytes())
} 