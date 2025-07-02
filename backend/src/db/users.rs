use rusqlite::{params, OptionalExtension, Result as RusqliteResult, Row};
use serde::{Serialize, Deserialize};
use crate::AppResult;
use super::DbManager;
use crate::error::AppError;
use crate::models::user::{User, Role};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::str::FromStr;

impl User {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            username: row.get(1)?,
            password_hash: row.get(2)?,
            role: Role::from_str(&row.get::<_, String>(3)?).unwrap_or(Role::Viewer),
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }
}

impl DbManager {
    pub async fn create_user(&self, username: &str, password: &str, email: &str, role: Role) -> Result<i64, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        let role_str = role.to_string();
        let username = username.to_string();
        let email = email.to_string();
        
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO users (username, password_hash, role, email, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![username, password_hash, role_str, email, chrono::Utc::now().to_rfc3339(), chrono::Utc::now().to_rfc3339()],
                )?;
                Ok(conn.last_insert_rowid())
            })
            .await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let username = username.to_string();
        self.conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT id, username, password_hash, role, created_at, updated_at FROM users WHERE username = ?1",
                    params![username],
                    User::from_row,
                )
                .optional()
            })
            .await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>, AppError> {
        self.conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT id, username, password_hash, role, created_at, updated_at FROM users WHERE id = ?1",
                    params![user_id],
                    User::from_row,
                )
                .optional()
            })
            .await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_password_hash(&self, username: &str) -> Result<Option<String>, AppError> {
        let username = username.to_string();
        self.conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT password_hash FROM users WHERE username = ?1",
                    params![username],
                    |row| row.get(0),
                )
                .optional()
            })
            .await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn update_password(&self, user_id: i64, new_password_hash: &str) -> Result<(), AppError> {
        let new_password_hash = new_password_hash.to_string();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE users SET password_hash = ?1 WHERE id = ?2",
                    params![new_password_hash, user_id],
                )
                .map(|_| ())
            })
            .await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare("SELECT id, username, password_hash, role, created_at, updated_at FROM users")?;
                let users_iter = stmt.query_map([], User::from_row)?;
                users_iter.collect::<RusqliteResult<Vec<User>>>()
            })
            .await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn update_user_role(&self, username: &str, new_role: &str) -> Result<bool, AppError> {
        let username_clone = username.to_string();
        let new_role_clone = new_role.to_string();
        self.conn.call(move |conn| {
            let rows = conn.execute(
                "UPDATE users SET role = ? WHERE username = ?",
                params![new_role_clone, username_clone],
            )?;
            Ok(rows > 0)
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn delete_user(&self, username: &str) -> Result<bool, AppError> {
        let username_clone = username.to_string();
        self.conn.call(move |conn| {
            let rows = conn.execute(
                "DELETE FROM users WHERE username = ?",
                params![username_clone],
            )?;
            Ok(rows > 0)
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn list_users(&self) -> Result<Vec<User>, AppError> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT id, username, password_hash, role, created_at, updated_at FROM users")?;
            let users_iter = stmt.query_map([], User::from_row)?;
            users_iter.collect::<RusqliteResult<Vec<User>>>()
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<Option<User>, AppError> {
        let user = self.get_user_by_username(username).await?;

        if let Some(user) = user {
            if verify_password(&user.password_hash, password) {
                Ok(Some(user))
            } else {
                Ok(None) // Invalid password
            }
        } else {
            Ok(None) // User not found
        }
    }

    pub async fn change_password(&self, user_id: i64, current_password: &str, new_password: &str) -> Result<(), AppError> {
        let user = self.get_user_by_id(user_id).await?.ok_or_else(|| AppError::Auth("User not found".to_string()))?;

        if !verify_password(&user.password_hash, current_password) {
            return Err(AppError::Auth("Invalid current password".to_string()));
        }

        let new_password_hash = hash_password(new_password)?;

        self.update_password(user_id, &new_password_hash).await
    }
}

fn verify_password(hash: &str, password: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(hash) {
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    } else {
        false
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(e.to_string()))
}