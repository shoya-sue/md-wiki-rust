use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use axum::{
    routing::get,
    Router,
    response::Html,
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use md_wiki_rust_backend::{AppState, routes, config};
use std::sync::Arc;

async fn health_check() -> &'static str {
    "OK"
}

async fn hello_world() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
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

    tracing::info!("Environment loaded");

    // アプリケーション設定の読み込み
    let config = config::Config::from_env();
    
    // ストレージディレクトリの作成
    let markdown_dir = PathBuf::from(&config.markdown_dir);
    std::fs::create_dir_all(&markdown_dir).unwrap_or_else(|e| {
        tracing::warn!("Failed to create markdown directory: {}", e);
    });

    // アプリケーション状態の初期化
    let state = AppState {
        db_manager: None, // 必要に応じて初期化
        git_repo: None,   // 必要に応じて初期化
        markdown_dir,
        config: config.clone(),
    };

    // Build application with routes
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 基本ルートとAPIルートを統合
    let base_routes = Router::new()
        .route("/health", get(health_check))
        .route("/", get(hello_world));
        
    // APIルーターの追加（コメントアウトして徐々に追加する）
    let app = base_routes
        // .merge(routes::create_router(state))
        .layer(cors);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Server started, listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}