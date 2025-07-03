use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::{LoginCredentials, UserRegistration, ChangePasswordRequest, User, Role};
use crate::auth;
use crate::models::user::verify_password;
use std::str::FromStr;

// ユーザー登録
pub async fn register_user(
    State(state): State<AppState>,
    Json(registration): Json<UserRegistration>,
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
    
    // 役割の設定（指定がなければデフォルトでViewer）
    let role = registration.role.as_deref().unwrap_or("viewer");
    let role_enum = Role::from_str(role).map_err(|e| (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
            "error": format!("Invalid role: {}", e)
        })),
    ))?;
    
    // ユーザー作成
    match db.create_user(
        &registration.username,
        &registration.password,
        &registration.email,
        role_enum,
    ).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            eprintln!("Registration error: {}", e);
            if e.to_string().contains("already exists") {
                return Err((
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({
                        "error": "Username or email already exists"
                    })),
                ));
            }
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to create user: {}", e)
                })),
            ))
        }
    }
}

// ログイン
pub async fn login(
    State(state): State<AppState>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
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
    
    // ユーザー認証
    match db.authenticate_user(&credentials.username, &credentials.password).await {
        Ok(Some(user)) => {
            // JWTトークン生成
            match crate::auth::create_token(user.id, &user.role.to_string()) {
                Ok(token) => {
                    let user_without_hash = User {
                        id: user.id,
                        username: user.username,
                        password_hash: "".to_string(), // パスワードハッシュは返さない
                        role: user.role,
                        created_at: user.created_at,
                        updated_at: user.updated_at,
                    };
                    
                    Ok(Json(serde_json::json!({
                        "token": token,
                        "user": user_without_hash
                    })))
                },
                Err(e) => {
                    eprintln!("Token creation error: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": format!("Failed to generate token: {}", e)
                        })),
                    ))
                }
            }
        },
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid username or password"
            })),
        )),
        Err(e) => {
            eprintln!("Authentication error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Authentication error: {}", e)
                })),
            ))
        }
    }
}

// パスワード変更
pub async fn change_password(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(request): Json<ChangePasswordRequest>,
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
    
    // パスワード変更
    match db.change_password(user_id, &request.current_password, &request.new_password).await {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to change password: {}", e)
            })),
        )),
    }
}

// ユーザー一覧取得（管理者用）
pub async fn get_all_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<serde_json::Value>)> {
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
    
    // すべてのユーザーを取得
    match db.get_all_users().await {
        Ok(users) => {
            // パスワードハッシュを除外
            let users_without_hash: Vec<User> = users.into_iter().map(|user| {
                User {
                    id: user.id,
                    username: user.username,
                    password_hash: "".to_string(),
                    role: user.role,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                }
            }).collect();
            
            Ok(Json(users_without_hash))
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get users: {}", e)
            })),
        )),
    }
}

// ユーザー取得
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
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
    
    // ユーザーを取得
    match db.get_user_by_id(user_id).await {
        Ok(Some(user)) => {
            // パスワードハッシュを除外
            let user_without_hash = User {
                id: user.id,
                username: user.username,
                password_hash: "".to_string(),
                role: user.role,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };
            
            Ok(Json(user_without_hash))
        },
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "User not found"
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to get user: {}", e)
            })),
        )),
    }
}

// 自分のユーザー情報を取得
pub async fn get_current_user(
    State(state): State<AppState>,
    claims: auth::Claims,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = claims.sub;
    
    get_user(State(state), Path(user_id)).await
} 