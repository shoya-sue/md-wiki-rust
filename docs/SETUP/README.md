# セットアップ手順

## 必要な環境

### 1. Rust環境
- Rust 1.73.0以上
- Cargo（Rustのパッケージマネージャー）

### 2. Node.js環境
- Node.js 18.0.0以上
- npm 8.0.0以上

### 3. 開発ツール
- Git 2.0.0以上
- SQLite 3.0.0以上
- OpenSSL開発パッケージ

## インストール手順

### 1. Rust環境のセットアップ

```bash
# Rustupのインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Rustのバージョン確認
rustc --version
cargo --version

# 必要に応じて安定版に更新
rustup update stable
rustup default stable
```

### 2. Node.js環境のセットアップ

```bash
# Node.jsとnpmのインストール（OS依存）
# 例：Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# バージョン確認
node --version
npm --version
```

### 3. 必要なパッケージのインストール

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  sqlite3 \
  libsqlite3-dev
```

#### RHEL/CentOS/AlmaLinux
```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install \
  openssl-devel \
  sqlite \
  sqlite-devel \
  pkg-config
```

#### macOS
```bash
brew install \
  openssl \
  sqlite \
  pkg-config
```

### 4. プロジェクトのセットアップ

```bash
# リポジトリのクローン
git clone https://github.com/yourusername/md-wiki-rust.git
cd md-wiki-rust

# バックエンドの依存関係インストール
cd backend
cargo build

# フロントエンドの依存関係インストール
cd ../frontend
npm install

# Markdownファイル用のGitリポジトリ初期化
cd ../storage
mkdir -p markdown_files
cd markdown_files
git init
```

## 開発環境の起動

### バックエンドの起動

```bash
cd backend
cargo run
```

### フロントエンドの起動

```bash
cd frontend
npm run tauri dev
```

## トラブルシューティング

### OpenSSLの問題

環境変数の設定が必要な場合：

```bash
export OPENSSL_DIR=/usr/local/opt/openssl  # macOSの場合
export OPENSSL_DIR=/usr                    # Linux系の場合
```

### SQLiteの問題

権限の確認：

```bash
# データベースディレクトリの作成と権限設定
mkdir -p storage
chmod 755 storage
```

### Rustのビルドエラー

キャッシュのクリーンアップ：

```bash
cargo clean
cargo update
cargo build
```

### Node.jsの依存関係エラー

npmキャッシュのクリア：

```bash
npm cache clean --force
rm -rf node_modules
npm install
```

## 開発リソース

- [Rust公式ドキュメント](https://www.rust-lang.org/ja/learn)
- [Axum公式ドキュメント](https://docs.rs/axum)
- [Tauri公式ドキュメント](https://tauri.app/v1/guides)
- [React公式ドキュメント](https://reactjs.org/docs/getting-started.html) 