use serde::{Deserialize, Serialize};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use rand::Rng;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

impl UserRole {
    pub fn has_permission(&self, required_role: &UserRole) -> bool {
        use UserRole::*;
        match (self, required_role) {
            (Admin, _) => true,
            (Editor, Editor | Viewer) => true,
            (Viewer, Viewer) => true,
            _ => false,
        }
    }

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

impl User {
    pub fn new(username: String, password: &str, role: UserRole) -> crate::AppResult<Self> {
        let password_hash = hash_password(password)?;
        Ok(User {
            id: 0, // This will be set by the database
            username,
            password_hash,
            email: String::new(),
            role,
            created_at: 0,
            last_login: None,
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        verify_password(&self.password_hash, password)
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

pub fn hash_password(password: &str) -> crate::AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| crate::AppError::Internal(e.to_string()))
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Editor,
    Viewer,
}

impl Role {
    pub fn can_edit(&self) -> bool {
        matches!(self, Role::Admin | Role::Editor)
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Editor => write!(f, "editor"),
            Role::Viewer => write!(f, "viewer"),
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "editor" => Ok(Role::Editor),
            "viewer" => Ok(Role::Viewer),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegistrationData {
    pub username: String,
    pub password: String,
    pub role: Role,
}

#[derive(Debug, Deserialize)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
} 