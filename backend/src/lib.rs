use std::fmt;
use thiserror::Error;

pub mod auth;
pub mod db;
pub mod git_ops;
pub mod handlers;
pub mod models;
pub mod routes;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Internal(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Internal(err.to_string())
    }
}

// Axumとの統合
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        use axum::Json;
        
        let (status, error_message) = match self {
            AppError::Authentication(_) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Authorization(_) => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "Not Found"),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "Bad Request"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
        };
        
        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "details": self.to_string()
            }
        }));
        
        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>; 