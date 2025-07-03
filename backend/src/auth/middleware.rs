use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::{
    error::AppError,
    models::user::Role,
    AppState,
};
use crate::auth;
use std::str::FromStr;

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(AppError::Auth("Missing authorization header".to_string()));
    };

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        let claims = auth::verify_token(token)?;
        req.extensions_mut().insert(claims);
        Ok(next.run(req).await)
    } else {
        Err(AppError::Auth("Invalid authorization header format".to_string()))
    }
}

pub async fn require_role(
    req: Request<axum::body::Body>,
    next: Next,
    required_role: Role,
) -> Result<Response, AppError> {
    let claims = req.extensions().get::<auth::Claims>()
        .ok_or_else(|| AppError::Auth("No claims found in request, ensure auth middleware is applied first".to_string()))?;

    let claims_role = Role::from_str(&claims.role).map_err(|_| AppError::Auth("Invalid role in token".to_string()))?;

    if claims_role >= required_role {
        Ok(next.run(req).await)
    } else {
        Err(AppError::Auth("Insufficient permissions".to_string()))
    }
}