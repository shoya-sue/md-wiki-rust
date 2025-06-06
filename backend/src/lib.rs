// ライブラリファイルを一時的に最小化して、コンパイルが通るようにする
use std::path::PathBuf;

// 最小限のモジュールのみをエクスポート
pub mod error;

#[derive(Clone, Debug)]
pub struct AppState {
    pub markdown_dir: PathBuf,
}

// error.rsへの参照型エイリアス
pub type AppResult<T> = Result<T, error::AppError>;