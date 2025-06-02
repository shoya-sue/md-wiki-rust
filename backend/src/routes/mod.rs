// ルーティングモジュール
// 将来的な拡張性のために空のモジュールを用意 

use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};

use crate::handlers::{
    document::{
        get_document,
        create_document,
        update_document,
        delete_document,
        get_document_history,
        get_document_version,
        search_documents,
        get_document_by_tag,
        get_all_tags,
        get_recent_documents,
    },
    auth::{
        register_user,
        login,
        change_password,
        get_all_users,
        get_user,
        get_current_user,
    },
};
use crate::auth::{auth_middleware, require_role};
use crate::models::UserRole;
use crate::AppState;

pub fn create_router(app_state: AppState) -> Router {
    // 認証が不要なルート
    let public_routes = Router::new()
        .route("/documents/:filename", get(get_document))
        .route("/documents/history/:filename", get(get_document_history))
        .route("/documents/version/:filename/:commit_id", get(get_document_version))
        .route("/search", get(search_documents))
        .route("/tags/:tag", get(get_document_by_tag))
        .route("/tags", get(get_all_tags))
        .route("/recent", get(get_recent_documents))
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login));
    
    // 認証が必要なルート（全ユーザー）
    let auth_routes = Router::new()
        .route("/auth/me", get(get_current_user))
        .route("/auth/password/:user_id", put(change_password))
        .layer(middleware::from_fn_with_state(app_state.clone(), auth_middleware));
    
    // エディター権限が必要なルート
    let editor_routes = Router::new()
        .route("/documents", post(create_document))
        .route("/documents/:filename", put(update_document))
        .layer(middleware::from_fn(require_role(UserRole::Editor)));
    
    // 管理者権限が必要なルート
    let admin_routes = Router::new()
        .route("/documents/:filename", delete(delete_document))
        .route("/auth/users", get(get_all_users))
        .route("/auth/users/:user_id", get(get_user))
        .layer(middleware::from_fn(require_role(UserRole::Admin)));
    
    // すべてのルートを結合
    Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(editor_routes)
        .merge(admin_routes)
        .with_state(app_state)
} 