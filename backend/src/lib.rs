use std::error::Error;
use std::fmt;

pub mod auth;
pub mod db;
pub mod git_ops;
pub mod handlers;
pub mod models;
pub mod routes;

#[derive(Debug)]
pub enum AppError {
    Database(rusqlite::Error),
    Authentication(String),
    Authorization(String),
    Git(git2::Error),
    IO(std::io::Error),
    NotFound(String),
    ValidationError(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Authentication(e) => write!(f, "Authentication error: {}", e),
            AppError::Authorization(e) => write!(f, "Authorization error: {}", e),
            AppError::Git(e) => write!(f, "Git error: {}", e),
            AppError::IO(e) => write!(f, "IO error: {}", e),
            AppError::NotFound(e) => write!(f, "Not found: {}", e),
            AppError::ValidationError(e) => write!(f, "Validation error: {}", e),
            AppError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<git2::Error> for AppError {
    fn from(err: git2::Error) -> Self {
        AppError::Git(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IO(err)
    }
}

pub type AppResult<T> = Result<T, AppError>; 