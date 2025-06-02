use rusqlite::{params, OptionalExtension};
use serde::{Serialize, Deserialize};
use crate::AppResult;
use super::DbManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

impl DbManager {
    pub fn create_user(&self, username: &str, password_hash: &str, role: &str) -> AppResult<i64> {
        let mut conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)",
            params![username, password_hash, role],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_user_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let mut conn = self.conn.lock().unwrap();
        let user = conn.query_row(
            "SELECT id, username, password_hash, role FROM users WHERE username = ?",
            params![username],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password_hash: row.get(2)?,
                    role: row.get(3)?,
                })
            }
        ).optional()?;
        Ok(user)
    }

    pub fn update_user_role(&self, username: &str, new_role: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE users SET role = ? WHERE username = ?",
            params![new_role, username],
        )?;
        Ok(rows > 0)
    }

    pub fn update_user_password(&self, username: &str, new_password_hash: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE users SET password_hash = ? WHERE username = ?",
            params![new_password_hash, username],
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