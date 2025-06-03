use std::path::Path;
use tokio_rusqlite::Connection;
use crate::error::AppError;

pub mod documents;
pub mod users;
pub mod tags;

#[derive(Clone)]
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
                conn.execute_batch(include_str!("schema.sql"))
                    .map_err(|e| AppError::Database(format!("Failed to initialize database: {}", e)))
            })
            .await
    }
}

pub use documents::DocumentMeta;
pub use users::User; 