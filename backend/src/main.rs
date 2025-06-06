use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // dotenvの使用はオプションに変更（環境変数が設定されていれば使用しない）
    if std::env::var("DATABASE_URL").is_err() {
        tracing::info!("Loading environment variables from .env file");
        dotenv::dotenv().ok();
    } else {
        tracing::info!("Using environment variables from docker-compose.yml");
    }

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
        .layer(cors);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000)); // 0.0.0.0に変更してコンテナ外からのアクセスを許可
    tracing::info!("listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}