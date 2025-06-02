use axum::{
    extract::{Path, State, Query},
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

#[derive(Serialize, Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    filename: String,
    content_preview: String,
    matches: usize,
}

#[derive(Serialize)]
pub struct SearchResults {
    results: Vec<SearchResult>,
    query: String,
    total_matches: usize,
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

// 検索機能の実装
pub async fn search_documents(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResults>, (StatusCode, Json<serde_json::Value>)> {
    let search_term = query.q.to_lowercase();
    if search_term.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Search query cannot be empty"
            })),
        ));
    }

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

    let mut results = Vec::new();
    let mut total_matches = 0;

    for entry in read_dir {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            let content_lower = content.to_lowercase();
                            if let Some(pos) = content_lower.find(&search_term) {
                                // Count occurrences
                                let matches_count = content_lower.matches(&search_term).count();
                                total_matches += matches_count;
                                
                                // Extract preview
                                let start = content_lower[..pos].rfind("\n").unwrap_or(0);
                                let end = content_lower[pos..].find("\n").map_or(content.len(), |p| pos + p);
                                let preview = content[start..end].trim().to_string();
                                
                                results.push(SearchResult {
                                    filename: file_name.to_string(),
                                    content_preview: preview,
                                    matches: matches_count,
                                });
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
    }

    // Sort by number of matches (descending)
    results.sort_by(|a, b| b.matches.cmp(&a.matches));

    Ok(Json(SearchResults {
        results,
        query: search_term,
        total_matches,
    }))
} 