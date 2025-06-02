use rusqlite::{params, OptionalExtension};
use serde::{Serialize, Deserialize};
use crate::AppResult;
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
    pub fn get_document_metadata(&self, filename: &str) -> AppResult<Option<DocumentMeta>> {
        let mut conn = self.conn.lock().unwrap();
        
        let document = conn.query_row(
            "SELECT id, filename, title, created_at, updated_at 
             FROM documents 
             WHERE filename = ?",
            params![filename],
            |row| {
                Ok(DocumentMeta {
                    id: row.get(0)?,
                    filename: row.get(1)?,
                    title: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    tags: Vec::new(),
                })
            }
        ).optional()?;
        
        if let Some(mut doc) = document {
            let mut stmt = conn.prepare(
                "SELECT t.name 
                 FROM tags t 
                 JOIN document_tags dt ON t.id = dt.tag_id 
                 WHERE dt.document_id = ?"
            )?;
            
            let tags: Vec<String> = stmt
                .query_map(params![doc.id], |row| row.get(0))?
                .collect::<Result<_, _>>()?;
            
            doc.tags = tags;
            Ok(Some(doc))
        } else {
            Ok(None)
        }
    }

    pub fn create_document_metadata(&self, filename: &str, title: Option<&str>) -> AppResult<i64> {
        let mut conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO documents (filename, title) VALUES (?, ?)",
            params![filename, title],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_document_metadata(&self, filename: &str, title: Option<&str>) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE documents SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE filename = ?",
            params![title, filename],
        )?;
        Ok(rows > 0)
    }

    pub fn delete_document_metadata(&self, filename: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        
        let document_id: Option<i64> = tx
            .query_row("SELECT id FROM documents WHERE filename = ?", params![filename], |row| {
                Ok(row.get(0)?)
            })
            .optional()?;
        
        if let Some(id) = document_id {
            tx.execute(
                "DELETE FROM document_tags WHERE document_id = ?",
                params![id],
            )?;
            
            tx.execute(
                "DELETE FROM documents WHERE id = ?",
                params![id],
            )?;
            
            tx.commit()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_documents(&self) -> AppResult<Vec<DocumentMeta>> {
        let mut conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, filename, title, created_at, updated_at FROM documents"
        )?;
        
        let docs = stmt.query_map([], |row| {
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
        for doc in docs {
            let mut document = doc?;
            let mut tag_stmt = conn.prepare(
                "SELECT t.name 
                 FROM tags t 
                 JOIN document_tags dt ON t.id = dt.tag_id 
                 WHERE dt.document_id = ?"
            )?;
            
            let tags: Vec<String> = tag_stmt
                .query_map(params![document.id], |row| row.get(0))?
                .collect::<Result<_, _>>()?;
            
            document.tags = tags;
            documents.push(document);
        }
        
        Ok(documents)
    }
} 