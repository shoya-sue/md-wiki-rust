use axum::{
    routing::{get, post},
    Router, Json,
    http::{StatusCode, Method},
};
use std::{net::SocketAddr, fs, path::PathBuf};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;
mod routes;
mod git_ops;
mod db;
mod models;
mod auth;

use db::DbManager;

#[derive(Clone)]
pub struct AppState {
    pub markdown_dir: PathBuf,
    pub db_manager: Option<DbManager>,
}

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Set up markdown directory
    let markdown_dir = PathBuf::from("../storage/markdown_files");
    if !markdown_dir.exists() {
        fs::create_dir_all(&markdown_dir).expect("Failed to create markdown directory");
    }
    
    // SQLiteデータベースの初期化
    let db_path = PathBuf::from("../storage/md_wiki.db");
    let db_manager = match DbManager::new(&db_path) {
        Ok(manager) => {
            tracing::info!("Database initialized successfully");
            Some(manager)
        },
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            None
        }
    };
    
    // Set up application state
    let app_state = AppState {
        markdown_dir,
        db_manager,
    };
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);
    
    // 新しいルーターを構築
    let router = routes::create_router(app_state);
    
    // CORSレイヤーを追加
    let app = router.layer(cors);
    
    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Simple health check endpoint
async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "ok" })),
    )
} 