use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::{
    error::AppError,
    models::Role,
    AppState,
};

// 認証ミドルウェア
pub async fn require_auth<S>(
    State(state): State<S>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| AppError::Auth("Missing authorization header".into()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::Auth("Invalid authorization header".into()));
    }

    let token = &auth_header["Bearer ".len()..];
    let claims = crate::auth::jwt::verify_token(token)
        .map_err(|_| AppError::Auth("Invalid token".into()))?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub async fn require_role<S>(
    State(state): State<S>,
    mut req: Request<Body>,
    next: Next,
    required_role: Role,
) -> Result<Response, AppError> {
    let claims = req
        .extensions()
        .get::<crate::auth::jwt::Claims>()
        .ok_or_else(|| AppError::Auth("Missing authentication".into()))?;

    if claims.role >= required_role {
        Ok(next.run(req).await)
    } else {
        Err(AppError::Auth("Insufficient permissions".into()))
    }
}

pub fn auth_middleware<S>(state: S) -> axum::middleware::from_fn::FromFn<S, require_auth<S>> {
    axum::middleware::from_fn_with_state(state, require_auth)
}

pub fn role_middleware<S>(state: S, role: Role) -> axum::middleware::from_fn::FromFn<S, require_role<S>> {
    axum::middleware::from_fn_with_state(state, move |state, req, next| require_role(state, req, next, role))
} 