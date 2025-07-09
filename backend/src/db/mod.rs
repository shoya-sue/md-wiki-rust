use std::path::Path;
use tokio_rusqlite::Connection;
use crate::error::AppError;

pub mod documents;
pub mod users;
pub mod tags;

#[derive(Clone, Debug)]
pub struct DbManager {
    conn: Connection,
}

impl DbManager {
    pub async fn new(db_path: &Path) -> Result<Self, AppError> {
        let conn = Connection::open(db_path)
            .await
            .map_err(|e| AppError::Database(format!("Failed to open database: {}", e)))?;

        Ok(Self { conn })
    }

    pub async fn init(&self) -> Result<(), AppError> {
        self.conn
            .call(|conn| {
                conn.execute_batch(include_str!("schema.sql"))?;
                Ok(())
            })
            .await
            .map_err(|e| AppError::Database(format!("Failed to initialize database: {}", e)))
    }
}

pub use crate::models::user::User;
pub use documents::{DocumentMeta, self as document_ops};
pub use tags::{Tag, self as tag_ops}; 