use rusqlite::{params, OptionalExtension, Row};
use serde::{Serialize, Deserialize};
use crate::AppResult;
use super::DbManager;
use crate::error::AppError;
use crate::models::user::{User, Role};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: Role,
    pub created_at: String,
    pub updated_at: String,
}

impl User {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            username: row.get(1)?,
            role: Role::from_str(&row.get::<_, String>(2)?).unwrap_or(Role::Viewer),
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }
}

impl DbManager {
    pub async fn create_user(&self, username: &str, password_hash: &str, role: &str) -> Result<i64, AppError> {
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, ?3)",
                    params![username, password_hash, role],
                )
                .and_then(|_| conn.last_insert_rowid())
                .map_err(AppError::Database)
            })
            .await
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        self.conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT id, username, role, created_at, updated_at FROM users WHERE username = ?1",
                    params![username],
                    User::from_row,
                )
                .optional()
                .map_err(AppError::Database)
            })
            .await
    }

    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>, AppError> {
        self.conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT id, username, role, created_at, updated_at FROM users WHERE id = ?1",
                    params![user_id],
                    User::from_row,
                )
                .optional()
                .map_err(AppError::Database)
            })
            .await
    }

    pub async fn get_password_hash(&self, username: &str) -> Result<Option<String>, AppError> {
        self.conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT password_hash FROM users WHERE username = ?1",
                    params![username],
                    |row| row.get(0),
                )
                .optional()
                .map_err(AppError::Database)
            })
            .await
    }

    pub async fn update_password(&self, user_id: i64, new_password_hash: &str) -> Result<(), AppError> {
        self.conn
            .call(move |conn| {
                conn.execute(
                    "UPDATE users SET password_hash = ?1 WHERE id = ?2",
                    params![new_password_hash, user_id],
                )
                .map(|_| ())
                .map_err(AppError::Database)
            })
            .await
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare("SELECT id, username, role, created_at, updated_at FROM users")?;
                let users = stmt
                    .query_map([], User::from_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(users)
            })
            .await
            .map_err(AppError::Database)
    }

    pub fn update_user_role(&self, username: &str, new_role: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE users SET role = ? WHERE username = ?",
            params![new_role, username],
        )?;
        Ok(rows > 0)
    }

    pub fn delete_user(&self, username: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM users WHERE username = ?",
            params![username],
        )?;
        Ok(rows > 0)
    }

    pub fn list_users(&self) -> AppResult<Vec<User>> {
        let mut conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, role FROM users"
        )?;
        
        let users = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                role: row.get(3)?,
            })
        })?;
        
        users.collect::<Result<_, _>>().map_err(Into::into)
    }
}

impl crate::db::Database {
    pub async fn create_user(&self, username: &str, password: &str, role: Role) -> Result<i64, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        let role_str = role.to_string();
        
        self.connection
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO users (username, password_hash, role) VALUES (?1, ?2, ?3)",
                    rusqlite::params![username, password_hash, role_str],
                )
                .map(|_| conn.last_insert_rowid())
                .map_err(AppError::Database)
            })
            .await
    }

    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<User, AppError> {
        let user = self.connection
            .call(move |conn| {
                conn.query_row(
                    "SELECT id, username, password_hash, role FROM users WHERE username = ?1",
                    [username],
                    |row| {
                        Ok(User {
                            id: row.get(0)?,
                            username: row.get(1)?,
                            password_hash: row.get(2)?,
                            role: Role::from_str(&row.get::<_, String>(3)?),
                        })
                    },
                )
                .map_err(AppError::Database)
            })
            .await?;

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            Ok(user)
        } else {
            Err(AppError::Auth("Invalid password".to_string()))
        }
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        self.connection
            .call(move |conn| {
                let mut stmt = conn.prepare("SELECT id, username, password_hash, role FROM users")?;
                let users = stmt
                    .query_map([], |row| {
                        Ok(User {
                            id: row.get(0)?,
                            username: row.get(1)?,
                            password_hash: row.get(2)?,
                            role: Role::from_str(&row.get::<_, String>(3)?),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(users)
            })
            .await
            .map_err(AppError::Database)
    }

    pub async fn change_password(&self, user_id: i64, current_password: &str, new_password: &str) -> Result<(), AppError> {
        let user = self.get_user_by_id(user_id).await?;

        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if Argon2::default()
            .verify_password(current_password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Err(AppError::Auth("Invalid current password".to_string()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let new_password_hash = argon2
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .to_string();

        self.connection
            .call(move |conn| {
                conn.execute(
                    "UPDATE users SET password_hash = ?1 WHERE id = ?2",
                    rusqlite::params![new_password_hash, user_id],
                )
                .map(|_| ())
                .map_err(AppError::Database)
            })
            .await
    }
} 