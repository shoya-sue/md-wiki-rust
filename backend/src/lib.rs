// 空のlib.rsファイル
// 必要な時点で機能を追加します

// MD-Wikiのライブラリモジュール
use std::path::PathBuf;


pub mod auth;
pub mod db;
pub mod error;
pub mod git_ops;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod config;

use db::DbManager;
use git_ops::GitRepository;

// エラー型のエクスポート
pub use error::{AppError, AppResult};

// アプリケーションの状態を保持する構造体
#[derive(Clone, Debug)]
pub struct AppState {
    pub db_manager: Option<DbManager>,
    pub git_repo: Option<GitRepository>,
    pub markdown_dir: PathBuf,
    pub config: config::Config,
}