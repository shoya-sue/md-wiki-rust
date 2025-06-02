// ルーティングモジュール
// 将来的な拡張性のために空のモジュールを用意 

use axum::{
    routing::{get, post, put, delete},
    Router,
};

use crate::handlers::{
    auth::{login, register, get_current_user},
    document::{
        get_document,
        save_document,
        list_documents,
        search_documents,
        get_document_history,
        get_document_version,
        delete_document,
    },
    metadata::{
        get_document_metadata,
        update_document_metadata,
        get_all_tags,
        get_document_by_tag,
        get_recent_documents,
    },
};
use crate::auth::middleware::{auth_layer, role_layer};
use crate::models::user::UserRole;
use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(get_current_user))
        .with_state(state.clone());

    let document_routes = Router::new()
        .route("/", get(list_documents).post(save_document))
        .route("/search", get(search_documents))
        .route("/:filename", 
            get(get_document)
                .put(save_document)
                .delete(delete_document)
        )
        .route("/:filename/history", get(get_document_history))
        .route("/:filename/version/:commit_id", get(get_document_version))
        .route("/:filename/metadata", 
            get(get_document_metadata)
                .put(update_document_metadata)
        )
        .layer(auth_layer())
        .with_state(state.clone());

    let tag_routes = Router::new()
        .route("/", get(get_all_tags))
        .route("/:tag/documents", get(get_document_by_tag))
        .layer(auth_layer())
        .with_state(state.clone());

    Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/documents", document_routes)
        .nest("/api/tags", tag_routes)
        .with_state(state)
} 