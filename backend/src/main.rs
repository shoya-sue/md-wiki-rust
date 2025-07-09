use std::net::SocketAddr;
use std::path::PathBuf;
use axum::{
    routing::get,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use md_wiki_backend::{AppState, routes, config};

async fn health_check() -> &'static str {
    "OK"
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

    // CORS設定
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 静的ファイル配信用のルーター
    // frontend/dist ディレクトリを配信する
    // このパスはDockerコンテナ内でのパスに合わせる必要がある
    let static_files_service = ServeDir::new("frontend/dist")
        .not_found_service(ServeDir::new("frontend/dist/index.html"));

    // APIルーターと静的ファイル配信を組み合わせる
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api", routes::create_router(state)) // APIルートを /api 以下にネスト
        .fallback_service(static_files_service) // APIにマッチしないものは静的ファイルとして配信
        .layer(cors);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("Server started, listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
