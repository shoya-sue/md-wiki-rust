use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use futures::future::BoxFuture;
use std::sync::Arc;
use crate::auth::jwt;
use crate::models::UserRole;
use crate::AppState;

// 認証ミドルウェア
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Authorizationヘッダーからトークンを取得
    let auth_header = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok());
    
    if let Some(auth_header) = auth_header {
        // "Bearer "プレフィックスを削除してトークンを取得
        if auth_header.starts_with("Bearer ") {
            let token = &auth_header[7..];
            
            // トークンを検証
            match jwt::verify_token(token) {
                Ok(claims) => {
                    // リクエストの拡張データにユーザー情報を追加
                    let mut request = request;
                    request.extensions_mut().insert(claims);
                    
                    // 次のハンドラーに処理を委譲
                    return Ok(next.run(request).await);
                },
                Err(e) => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(serde_json::json!({
                            "error": format!("Invalid token: {}", e)
                        })),
                    ));
                }
            }
        }
    }
    
    Err((
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "error": "Authorization header missing or invalid"
        })),
    ))
}

// ロールベースのアクセス制御ミドルウェア
pub fn require_role(role: UserRole) -> impl Fn(Request, Next) -> BoxFuture<'static, Response> {
    move |request: Request, next: Next| {
        let fut = async move {
            // リクエストの拡張データからJWTクレームを取得
            if let Some(claims) = request.extensions().get::<jwt::Claims>() {
                // ユーザーロールを比較
                let user_role = UserRole::from_str(&claims.role);
                
                // Admin > Editor > Viewer の順で権限が高い
                let has_permission = match role {
                    UserRole::Admin => user_role == UserRole::Admin,
                    UserRole::Editor => user_role == UserRole::Admin || user_role == UserRole::Editor,
                    UserRole::Viewer => true, // すべてのロールがViewerの権限を持つ
                };
                
                if has_permission {
                    // 権限があれば次のハンドラーに処理を委譲
                    return next.run(request).await;
                }
                
                // 権限がなければ403エラー
                return Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .header("Content-Type", "application/json")
                    .body(
                        serde_json::to_string(&serde_json::json!({
                            "error": "Insufficient permissions"
                        }))
                        .unwrap()
                        .into(),
                    )
                    .unwrap();
            }
            
            // クレームがなければ認証エラー（auth_middlewareを通過していないか、リクエストが改ざんされている）
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("Content-Type", "application/json")
                .body(
                    serde_json::to_string(&serde_json::json!({
                        "error": "Authentication required"
                    }))
                    .unwrap()
                    .into(),
                )
                .unwrap()
        };
        
        Box::pin(fut)
    }
} 