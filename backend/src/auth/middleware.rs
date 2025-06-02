use axum::{
    extract::State,
    middleware::Next,
    response::Response,
    http::{Request, StatusCode},
    response::IntoResponse,
};
use crate::{
    AppState,
    AppError,
    auth::jwt::{self, Claims},
    models::user::UserRole,
};

// 認証ミドルウェア
pub async fn require_auth<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Authentication("No authorization header".into()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Authentication("Invalid authorization header".into()));
    }

    let token = &auth_header["Bearer ".len()..];
    let claims = jwt::verify_token(token)
        .map_err(|e| AppError::Authentication(e.to_string()))?;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

pub async fn require_role<B>(
    required_role: UserRole,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::Authorization("No claims found".into()))?;

    let user_role = UserRole::from_str(&claims.role);
    if !user_role.has_permission(&required_role) {
        return Err(AppError::Authorization("Insufficient permissions".into()));
    }

    Ok(next.run(request).await)
}

pub fn auth_layer<B>() -> axum::middleware::from_fn::FromFn<impl Fn(Request<B>, Next<B>) -> impl std::future::Future<Output = Response> + Clone> {
    axum::middleware::from_fn(require_auth)
}

pub fn role_layer<B>(role: UserRole) -> axum::middleware::from_fn::FromFn<impl Fn(Request<B>, Next<B>) -> impl std::future::Future<Output = Response> + Clone> {
    axum::middleware::from_fn(move |req, next| require_role(role, req, next))
} 