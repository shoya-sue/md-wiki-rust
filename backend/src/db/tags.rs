use rusqlite::{params, OptionalExtension, Result as RusqliteResult};
use serde::{Serialize, Deserialize};
use crate::error::AppError;
use super::DbManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

impl DbManager {
    pub async fn create_tag(&self, name: &str) -> Result<i64, AppError> {
        let name_clone = name.to_string();
        self.conn.call(move |conn| {
            conn.execute("INSERT INTO tags (name) VALUES (?)", params![name_clone])?;
            Ok(conn.last_insert_rowid())
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>, AppError> {
        let name_clone = name.to_string();
        self.conn.call(move |conn| {
            conn.query_row(
                "SELECT id, name FROM tags WHERE name = ?",
                params![name_clone],
                |row| {
                    Ok(Tag {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                },
            ).optional()
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn list_tags(&self) -> Result<Vec<Tag>, AppError> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT id, name FROM tags")?;
            let tags_iter = stmt.query_map([], |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })?;
            tags_iter.collect::<RusqliteResult<Vec<Tag>>>()
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn add_tag_to_document(&self, document_id: i64, tag_name: &str) -> Result<bool, AppError> {
        let tag_name_clone = tag_name.to_string();
        self.conn.call(move |conn| {
            let tx = conn.transaction()?;
            
            let tag_id = match tx.query_row("SELECT id FROM tags WHERE name = ?", params![&tag_name_clone], |row| row.get::<_, i64>(0)).optional()? {
                Some(id) => id,
                None => {
                    tx.execute("INSERT INTO tags (name) VALUES (?)", params![&tag_name_clone])?;
                    tx.last_insert_rowid()
                }
            };
            
            tx.execute("INSERT OR IGNORE INTO document_tags (document_id, tag_id) VALUES (?, ?)", params![document_id, tag_id])?;
            tx.commit()?;
            Ok(true)
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn remove_tag_from_document(&self, document_id: i64, tag_name: &str) -> Result<bool, AppError> {
        let tag_name_clone = tag_name.to_string();
        self.conn.call(move |conn| {
            if let Some(tag_id) = conn.query_row("SELECT id FROM tags WHERE name = ?", params![&tag_name_clone], |row| row.get(0)).optional()? {
                let rows_affected = conn.execute("DELETE FROM document_tags WHERE document_id = ? AND tag_id = ?", params![document_id, tag_id])?;
                Ok(rows_affected > 0)
            } else {
                Ok(false)
            }
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_documents_by_tag(&self, tag_name: &str) -> Result<Vec<String>, AppError> {
        let tag_name_clone = tag_name.to_string();
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT d.filename FROM documents d 
                 JOIN document_tags dt ON d.id = dt.document_id 
                 JOIN tags t ON dt.tag_id = t.id 
                 WHERE t.name = ?"
            )?;
            let filenames_iter = stmt.query_map(params![tag_name_clone], |row| row.get(0))?;
            filenames_iter.collect::<RusqliteResult<Vec<String>>>()
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }
}