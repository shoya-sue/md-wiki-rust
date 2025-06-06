use std::net::SocketAddr;
use axum::{
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use dotenv::dotenv;

mod config;
mod error;

// 一時的にコメントアウト
// mod db;
// mod api;
// mod models;
// mod utils;

#[tokio::main]
async fn main() {    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 環境変数の読み込み（Dockerから設定されている場合は無視される）
    dotenv().ok();
    tracing::info!("Environment loaded");

    // 一時的にデータベース初期化をコメントアウト
    // let db = db::init_db().await.expect("Failed to initialize database");

    // Build application with minimal routes
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
    // .with_state(db) // 一時的にコメントアウト
        .layer(cors);    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000)); // 0.0.0.0に変更してコンテナ外からのアクセスを許可
    tracing::info!("listening on {}", addr);
    
    // Axum 0.7ではhyperを直接使用する必要がある
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Server started, listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}