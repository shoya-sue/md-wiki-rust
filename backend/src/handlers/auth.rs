use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::models::{LoginCredentials, UserRegistration, ChangePasswordRequest, User};
use crate::auth::jwt;

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
    
    // ユーザー作成
    match db.create_user(
        &registration.username,
        &registration.password,
        &registration.email,
        role,
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
            match jwt::generate_token(&user) {
                Ok(token) => {
                    let user_without_hash = User {
                        id: user.id,
                        username: user.username,
                        password_hash: "".to_string(), // パスワードハッシュは返さない
                        email: user.email,
                        role: user.role,
                        created_at: user.created_at,
                        last_login: user.last_login,
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
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Current password is incorrect"
            })),
        )),
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
                    email: user.email,
                    role: user.role,
                    created_at: user.created_at,
                    last_login: user.last_login,
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
                email: user.email,
                role: user.role,
                created_at: user.created_at,
                last_login: user.last_login,
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
    claims: jwt::Claims,
) -> Result<Json<User>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = claims.sub.parse::<i64>().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Invalid user ID in token"
            })),
        )
    })?;
    
    get_user(State(state), Path(user_id)).await
} 