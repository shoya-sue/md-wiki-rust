use axum::{
    routing::{get, post},
    Router,
    middleware,
};

use crate::handlers::{
    auth::{login, register_user, get_current_user},
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
        search_documents_by_tag,
    },
};
use crate::auth::middleware::{require_auth};
use crate::AppState;

pub fn create_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login))
        .route("/me", get(get_current_user).route_layer(middleware::from_fn_with_state(state.clone(), require_auth)))
        .with_state(state.clone());

    let document_routes = Router::new()
        .route("/", get(list_documents).post(save_document))
        .route("/search", get(search_documents))
        .route("/recent", get(list_recent_documents))
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
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .with_state(state.clone());

    let tag_routes = Router::new()
        .route("/", get(get_all_tags))
        .route("/:tag/documents", get(search_documents_by_tag))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth))
        .with_state(state.clone());

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/documents", document_routes)
        .nest("/tags", tag_routes)
} 