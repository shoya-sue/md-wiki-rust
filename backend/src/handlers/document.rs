use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::AppState;
use crate::git_ops::{GitRepository, CommitInfo};

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

#[derive(Serialize)]
pub struct DocumentHistory {
    filename: String,
    commits: Vec<CommitInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct DocumentVersion {
    filename: String,
    content: String,
    commit_info: CommitInfo,
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
        Ok(_) => {
            // Gitコミットを作成
            let git_repo = match GitRepository::new(&state.markdown_dir) {
                Ok(ops) => ops,
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to initialize Git operations: {}", e)
                        })),
                    ));
                }
            };
            
            let commit_message = format!("Update {}.md", filename);
            match git_repo.commit_file(&format!("{}.md", filename), &document.content, &commit_message) {
                Ok(_) => Ok(StatusCode::OK),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to commit document: {}", e)
                    })),
                )),
            }
        },
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

#[derive(Deserialize)]
pub struct RecentDocumentsQuery {
    limit: Option<u32>,
}

#[derive(Serialize)]
pub struct RecentDocument {
    id: i64,
    filename: String,
    title: String,
    created_at: String,
    updated_at: String,
    view_count: u32,
    tags: Vec<String>,
}

#[derive(Serialize)]
pub struct RecentDocumentsResponse {
    documents: Vec<RecentDocument>,
}

pub async fn list_recent_documents(
    State(state): State<AppState>,
    Query(query): Query<RecentDocumentsQuery>,
) -> Result<Json<RecentDocumentsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let limit = query.limit.unwrap_or(10);

    let db_manager = state.db_manager.ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "Database not initialized" })))
    })?;

    match db_manager.list_recent_documents_meta(limit).await {
        Ok(docs_meta) => {
            let documents: Vec<RecentDocument> = docs_meta.into_iter().map(|meta| {
                RecentDocument {
                    id: meta.id,
                    filename: meta.filename,
                    title: meta.title.unwrap_or_else(|| "Untitled".to_string()),
                    created_at: meta.created_at,
                    updated_at: meta.updated_at,
                    view_count: 0, // DBにview_countがないため、0を返す
                    tags: meta.tags,
                }
            }).collect();
            Ok(Json(RecentDocumentsResponse { documents }))
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to fetch recent documents: {}", e)
            })),
        )),
    }
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

// ドキュメントの変更履歴を取得
pub async fn get_document_history(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<DocumentHistory>, (StatusCode, Json<serde_json::Value>)> {
    let file_path = format!("{}.md", filename);
    
    let git_repo = match GitRepository::new(&state.markdown_dir) {
        Ok(ops) => ops,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to initialize Git operations: {}", e)
                })),
            ));
        }
    };
    
    match git_repo.get_file_history(&file_path) {
        Ok(commits) => Ok(Json(DocumentHistory {
            filename,
            commits,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get document history: {}", e)
            })),
        )),
    }
}

// 特定バージョンのドキュメントを取得
pub async fn get_document_version(
    State(state): State<AppState>,
    Path((filename, commit_id)): Path<(String, String)>,
) -> Result<Json<DocumentVersion>, (StatusCode, Json<serde_json::Value>)> {
    let file_path = format!("{}.md", filename);
    
    let git_repo = match GitRepository::new(&state.markdown_dir) {
        Ok(ops) => ops,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to initialize Git operations: {}", e)
                })),
            ));
        }
    };
    
    // 履歴から特定のコミット情報を取得
    let history = match git_repo.get_file_history(&file_path) {
        Ok(commits) => commits,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to get document history: {}", e)
                })),
            ));
        }
    };
    
    let commit_info = match history.iter().find(|commit| commit.id.starts_with(&commit_id)) {
        Some(commit) => commit.clone(),
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Commit {} not found for document {}", commit_id, filename)
                })),
            ));
        }
    };
    
    // 特定バージョンの内容を取得
    match git_repo.get_file_content_at_commit(&file_path, &commit_info.id) {
        Ok(content) => Ok(Json(DocumentVersion {
            filename,
            content,
            commit_info,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get document at commit {}: {}", commit_id, e)
            })),
        )),
    }
}

// ドキュメントを削除する
pub async fn delete_document(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let file_path = state.markdown_dir.join(format!("{}.md", filename));
    
    // ファイルが存在するか確認
    if !file_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Document {} not found", filename)
            })),
        ));
    }
    
    // ファイルを削除
    match fs::remove_file(&file_path) {
        Ok(_) => {
            // Gitオペレーションを初期化
            let git_repo = match GitRepository::new(&state.markdown_dir) {
                Ok(ops) => ops,
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to initialize Git operations: {}", e)
                        })),
                    ));
                }
            };
            
            // ファイル削除をコミット
            let commit_message = format!("Delete {}.md", filename);
            match git_repo.remove_file(&format!("{}.md", filename), &commit_message) {
                Ok(_) => {
                    // データベースからメタデータも削除
                    if let Some(db) = &state.db_manager {
                        // エラーが発生しても処理は続行（ファイルは削除済み）
                        if let Err(e) = db.delete_document_metadata(&filename).await {
                            tracing::warn!("Failed to delete metadata for {}: {}", filename, e);
                        }
                    }
                    
                    Ok(StatusCode::OK)
                },
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to commit document deletion: {}", e)
                    })),
                )),
            }
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to delete document: {}", e)
            })),
        )),
    }
}

// 新しいドキュメントを作成
pub async fn create_document(
    State(state): State<AppState>,
    Json(document): Json<Document>,
) -> Result<Json<Document>, (StatusCode, Json<serde_json::Value>)> {
    let filename = document.filename.clone();
    let file_path = state.markdown_dir.join(format!("{}.md", filename));
    
    // 同名のファイルが既に存在するか確認
    if file_path.exists() {
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": format!("Document {} already exists", filename)
            })),
        ));
    }
    
    // ファイルを作成
    match fs::write(&file_path, &document.content) {
        Ok(_) => {
            // Gitコミットを作成
            let git_repo = match GitRepository::new(&state.markdown_dir) {
                Ok(ops) => ops,
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to initialize Git operations: {}", e)
                        })),
                    ));
                }
            };
            
            let commit_message = format!("Create {}.md", filename);
            match git_repo.commit_file(&format!("{}.md", filename), &document.content, &commit_message) {
                Ok(_) => Ok(Json(document)),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to commit document: {}", e)
                    })),
                )),
            }
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to create document: {}", e)
            })),
        )),
    }
}

// ドキュメントを更新
pub async fn update_document(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(document): Json<Document>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let file_path = state.markdown_dir.join(format!("{}.md", filename));
    
    // ファイルが存在するか確認
    if !file_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Document {} not found", filename)
            })),
        ));
    }
    
    // ファイルを更新
    match fs::write(&file_path, &document.content) {
        Ok(_) => {
            // Gitコミットを作成
            let git_repo = match GitRepository::new(&state.markdown_dir) {
                Ok(ops) => ops,
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to initialize Git operations: {}", e)
                        })),
                    ));
                }
            };
            
            let commit_message = format!("Update {}.md", filename);
            match git_repo.commit_file(&format!("{}.md", filename), &document.content, &commit_message) {
                Ok(_) => Ok(StatusCode::OK),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to commit document: {}", e)
                    })),
                )),
            }
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to update document: {}", e)
            })),
        )),
    }
}