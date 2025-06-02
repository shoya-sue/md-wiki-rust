use rusqlite::{params, OptionalExtension};
use serde::{Serialize, Deserialize};
use crate::AppResult;
use super::DbManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

impl DbManager {
    pub fn create_tag(&self, name: &str) -> AppResult<i64> {
        let mut conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tags (name) VALUES (?)",
            params![name],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_tag_by_name(&self, name: &str) -> AppResult<Option<Tag>> {
        let mut conn = self.conn.lock().unwrap();
        let tag = conn.query_row(
            "SELECT id, name FROM tags WHERE name = ?",
            params![name],
            |row| {
                Ok(Tag {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            }
        ).optional()?;
        Ok(tag)
    }

    pub fn list_tags(&self) -> AppResult<Vec<Tag>> {
        let mut conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM tags")?;
        
        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        
        tags.collect::<Result<_, _>>().map_err(Into::into)
    }

    pub fn add_tag_to_document(&self, document_id: i64, tag_name: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        
        // Get or create tag
        let tag_id = match self.get_tag_by_name(tag_name)? {
            Some(tag) => tag.id,
            None => self.create_tag(tag_name)?,
        };
        
        // Add relation
        tx.execute(
            "INSERT OR IGNORE INTO document_tags (document_id, tag_id) VALUES (?, ?)",
            params![document_id, tag_id],
        )?;
        
        tx.commit()?;
        Ok(true)
    }

    pub fn remove_tag_from_document(&self, document_id: i64, tag_name: &str) -> AppResult<bool> {
        let mut conn = self.conn.lock().unwrap();
        let tag = self.get_tag_by_name(tag_name)?;
        
        if let Some(tag) = tag {
            let rows = conn.execute(
                "DELETE FROM document_tags WHERE document_id = ? AND tag_id = ?",
                params![document_id, tag.id],
            )?;
            Ok(rows > 0)
        } else {
            Ok(false)
        }
    }

    pub fn get_documents_by_tag(&self, tag_name: &str) -> AppResult<Vec<String>> {
        let mut conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT d.filename 
             FROM documents d 
             JOIN document_tags dt ON d.id = dt.document_id 
             JOIN tags t ON dt.tag_id = t.id 
             WHERE t.name = ?"
        )?;
        
        let filenames = stmt.query_map(params![tag_name], |row| row.get(0))?;
        filenames.collect::<Result<_, _>>().map_err(Into::into)
    }
} 