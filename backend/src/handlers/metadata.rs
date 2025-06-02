use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::AppState;
use crate::db::{DocumentMeta};

#[derive(Serialize, Deserialize)]
pub struct MetadataRequest {
    title: String,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TagsResponse {
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RecentDocumentsResponse {
    documents: Vec<DocumentMeta>,
}

// ドキュメントのメタデータを取得
pub async fn get_document_metadata(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Result<Json<DocumentMeta>, (StatusCode, Json<serde_json::Value>)> {
    let db = match &state.db_manager {
        Some(db) => db.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database not initialized"
                })),
            ));
        }
    };
    
    // 閲覧回数をインクリメント
    if let Err(e) = db.increment_view_count(&filename) {
        eprintln!("Failed to increment view count: {}", e);
        // エラーは無視して続行
    }
    
    // メタデータを取得
    match db.get_document_meta(&filename) {
        Ok(Some(meta)) => Ok(Json(meta)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": format!("Metadata for document {} not found", filename)
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get document metadata: {}", e)
            })),
        )),
    }
}

// ドキュメントのメタデータを更新
pub async fn update_document_metadata(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Json(meta_request): Json<MetadataRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let db = match &state.db_manager {
        Some(db) => db.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database not initialized"
                })),
            ));
        }
    };
    
    // 現在のUNIXタイムスタンプを取得
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    // 既存のメタデータを取得または新規作成
    let meta = match db.get_document_meta(&filename) {
        Ok(Some(mut existing_meta)) => {
            existing_meta.title = meta_request.title;
            existing_meta.tags = meta_request.tags;
            existing_meta.updated_at = now;
            existing_meta
        },
        Ok(None) | Err(_) => {
            // 新規作成（エラーが発生した場合も新規作成として扱う）
            DocumentMeta {
                id: 0, // 自動割り当て
                filename: filename.clone(),
                title: meta_request.title,
                created_at: now,
                updated_at: now,
                view_count: 0,
                tags: meta_request.tags,
            }
        }
    };
    
    // メタデータを保存
    match db.save_document_meta(&meta) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to save document metadata: {}", e)
            })),
        )),
    }
}

// すべてのタグを取得
pub async fn get_all_tags(
    State(state): State<AppState>,
) -> Result<Json<TagsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let db = match &state.db_manager {
        Some(db) => db.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database not initialized"
                })),
            ));
        }
    };
    
    match db.get_all_tags() {
        Ok(tags) => Ok(Json(TagsResponse { tags })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get tags: {}", e)
            })),
        )),
    }
}

// タグでドキュメントを検索
pub async fn search_documents_by_tag(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<RecentDocumentsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let db = match &state.db_manager {
        Some(db) => db.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database not initialized"
                })),
            ));
        }
    };
    
    let tag = params.get("tag").cloned().unwrap_or_default();
    if tag.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Tag parameter is required"
            })),
        ));
    }
    
    match db.search_documents_by_tag(&tag) {
        Ok(documents) => Ok(Json(RecentDocumentsResponse { documents })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to search documents by tag: {}", e)
            })),
        )),
    }
}

// 最近更新されたドキュメントを取得
pub async fn get_recent_documents(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<RecentDocumentsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let db = match &state.db_manager {
        Some(db) => db.clone(),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database not initialized"
                })),
            ));
        }
    };
    
    // limitパラメータ解析、デフォルトは10
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(10);
    
    match db.get_recent_documents(limit) {
        Ok(documents) => Ok(Json(RecentDocumentsResponse { documents })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get recent documents: {}", e)
            })),
        )),
    }
} 