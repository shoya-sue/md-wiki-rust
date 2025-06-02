use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct Document {
    filename: String,
    content: String,
}

#[derive(Serialize)]
pub struct DocumentList {
    documents: Vec<String>,
}

// Get a specific markdown document
pub async fn get_document(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let file_path = state.markdown_dir.join(format!("{}.md", filename));
    
    match fs::read_to_string(&file_path) {
        Ok(content) => Ok(Json(Document {
            filename,
            content,
        })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Document {} not found", filename)
            })),
        )),
    }
}

// Save a markdown document
pub async fn save_document(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(document): Json<Document>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let file_path = state.markdown_dir.join(format!("{}.md", filename));
    
    match fs::write(&file_path, &document.content) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to save document: {}", e)
            })),
        )),
    }
}

// List all available markdown documents
pub async fn list_documents(
    State(state): State<AppState>,
) -> Result<Json<DocumentList>, (StatusCode, Json<serde_json::Value>)> {
    let read_dir = match fs::read_dir(&state.markdown_dir) {
        Ok(dir) => dir,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to read markdown directory: {}", e)
                })),
            ));
        }
    };
    
    let mut documents = Vec::new();
    
    for entry in read_dir {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".md") {
                    documents.push(file_name.trim_end_matches(".md").to_string());
                }
            }
        }
    }
    
    Ok(Json(DocumentList { documents }))
} 