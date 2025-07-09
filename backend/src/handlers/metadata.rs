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
    
    // メタデータを取得
    match db.get_document_metadata(&filename).await {
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
    
    let _now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    
    // 既存のメタデータを取得または新規作成
    match db.get_document_metadata(&filename).await {
        Ok(Some(_existing_meta)) => {
            // 既存のメタデータを更新
            match db.update_document_metadata(&filename, Some(&meta_request.title)).await {
                Ok(_) => Ok(StatusCode::OK),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to update document metadata: {}", e)
                    })),
                )),
            }
        },
        Ok(None) | Err(_) => {
            // 新規作成
            match db.create_document_metadata(&filename, Some(&meta_request.title)).await {
                Ok(_) => Ok(StatusCode::CREATED),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Failed to create document metadata: {}", e)
                    })),
                )),
            }
        }
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
    
    match db.list_tags().await {
        Ok(tags) => Ok(Json(TagsResponse { tags: tags.into_iter().map(|tag| tag.name).collect() })),
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
) -> Result<Json<Vec<String>>, (StatusCode, Json<serde_json::Value>)> {
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
    
    match db.get_documents_by_tag(&tag).await {
        Ok(documents) => Ok(Json(documents)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to search documents by tag: {}", e)
            })),
        )),
    }
}