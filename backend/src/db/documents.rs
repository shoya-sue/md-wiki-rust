use rusqlite::{params, OptionalExtension, Result as RusqliteResult};
use serde::{Serialize, Deserialize};
use crate::error::AppError;
use super::DbManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMeta {
    pub id: i64,
    pub filename: String,
    pub title: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<String>,
}

impl DbManager {
    pub async fn get_document_metadata(&self, filename: &str) -> Result<Option<DocumentMeta>, AppError> {
        let filename_clone = filename.to_string();
        self.conn.call(move |conn| {
            let mut doc_meta: Option<DocumentMeta> = conn.query_row(
                "SELECT id, filename, title, created_at, updated_at FROM documents WHERE filename = ?",
                params![filename_clone],
                |row| {
                    Ok(DocumentMeta {
                        id: row.get(0)?,
                        filename: row.get(1)?,
                        title: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                        tags: Vec::new(),
                    })
                },
            ).optional()?;

            if let Some(ref mut doc) = doc_meta {
                let mut stmt = conn.prepare("SELECT t.name FROM tags t JOIN document_tags dt ON t.id = dt.tag_id WHERE dt.document_id = ?")?;
                let tags = stmt.query_map(params![doc.id], |row| row.get(0))?.collect::<RusqliteResult<Vec<String>>>()?;
                doc.tags = tags;
            }

            Ok(doc_meta)
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn create_document_metadata(&self, filename: &str, title: Option<&str>) -> Result<i64, AppError> {
        let filename_clone = filename.to_string();
        let title_clone = title.map(|s| s.to_string());
        self.conn.call(move |conn| {
            conn.execute(
                "INSERT INTO documents (filename, title) VALUES (?, ?)",
                params![filename_clone, title_clone],
            )?;
            Ok(conn.last_insert_rowid())
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn update_document_metadata(&self, filename: &str, title: Option<&str>) -> Result<bool, AppError> {
        let filename_clone = filename.to_string();
        let title_clone = title.map(|s| s.to_string());
        self.conn.call(move |conn| {
            let rows = conn.execute(
                "UPDATE documents SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE filename = ?",
                params![title_clone, filename_clone],
            )?;
            Ok(rows > 0)
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn delete_document_metadata(&self, filename: &str) -> Result<bool, AppError> {
        let filename_clone = filename.to_string();
        self.conn.call(move |conn| {
            let tx = conn.transaction()?;
            
            let document_id: Option<i64> = tx.query_row("SELECT id FROM documents WHERE filename = ?", params![filename_clone], |row| row.get(0)).optional()?;
            
            if let Some(id) = document_id {
                tx.execute("DELETE FROM document_tags WHERE document_id = ?", params![id])?;
                tx.execute("DELETE FROM documents WHERE id = ?", params![id])?;
                tx.commit()?;
                Ok(true)
            } else {
                Ok(false)
            }
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn list_documents(&self) -> Result<Vec<DocumentMeta>, AppError> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare("SELECT id, filename, title, created_at, updated_at FROM documents")?;
            
            let docs_iter = stmt.query_map([], |row| {
                Ok(DocumentMeta {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    title: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    tags: Vec::new(),
                })
            })?;

            let mut documents = Vec::new();
            for doc_result in docs_iter {
                let mut doc = doc_result?;
                let mut tag_stmt = conn.prepare("SELECT t.name FROM tags t JOIN document_tags dt ON t.id = dt.tag_id WHERE dt.document_id = ?")?;
                let tags = tag_stmt.query_map(params![doc.id], |row| row.get(0))?.collect::<RusqliteResult<Vec<String>>>()?;
                doc.tags = tags;
                documents.push(doc);
            }
            
            Ok(documents)
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn list_recent_documents_meta(&self, limit: u32) -> Result<Vec<DocumentMeta>, AppError> {
        self.conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id, filename, title, created_at, updated_at FROM documents ORDER BY updated_at DESC LIMIT ?"
            )?;
            
            let docs_iter = stmt.query_map(params![limit], |row| {
                Ok(DocumentMeta {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    title: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    tags: Vec::new(),
                })
            })?;

            let mut documents = Vec::new();
            for doc_result in docs_iter {
                let mut doc = doc_result?;
                let mut tag_stmt = conn.prepare("SELECT t.name FROM tags t JOIN document_tags dt ON t.id = dt.tag_id WHERE dt.document_id = ?")?;
                let tags = tag_stmt.query_map(params![doc.id], |row| row.get(0))?.collect::<RusqliteResult<Vec<String>>>()?;
                doc.tags = tags;
                documents.push(doc);
            }
            
            Ok(documents)
        }).await.map_err(|e| AppError::Database(e.to_string()))
    }
}