# RustWiki

## 概要

Rustを用いてMarkdownファイルを共有・編集できるGUIベースのWikiサービスを構築します。

## プロジェクト名

`md-wiki-rust`

## プロジェクトの目的

* Markdownファイルを共同で編集・共有
* GUIベースで直感的な操作
* 安全で高速な動作

## 技術スタック

### フロントエンド（GUI）

* **Tauri**

  * 軽量、高速で安全なRustベースのElectron代替ツール。
  * Reactをフロントエンドフレームワークとして使用。

### バックエンド（Rust API）

* **Axum**

  * シンプルで直感的、高速で非同期処理対応のRustフレームワーク。
* **pulldown-cmark**

  * Markdownを解析してHTMLに変換するためのライブラリ。

### データストア

* Markdownファイルのバージョン管理はGitを使用。
* SQLiteを使用してメタデータ管理。

## リポジトリ構造

```plaintext
md-wiki-rust/
├── frontend
│   ├── src
│   │   ├── components
│   │   ├── App.jsx
│   │   ├── main.jsx
│   │   └── styles.css
│   ├── src-tauri
│   │   ├── src
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── tauri.conf.json
│   ├── index.html
│   └── package.json
├── backend
│   ├── src
│   │   ├── routes
│   │   ├── handlers
│   │   ├── models
│   │   ├── auth
│   │   └── main.rs
│   └── Cargo.toml
├── storage
│   └── markdown_files (ここにMarkdownファイルを保存,このRespositoryはGitで管理されます。)
├── docs
│   ├── API
│   │   └── README.md (API仕様書)
│   ├── DEVELOPMENT
│   │   └── README.md (開発計画)
│   └── SETUP
│       └── README.md (セットアップ手順)
├── README.md
└── .gitignore
```

## 現在実装されている機能

* Markdownファイル作成・編集
* Markdownプレビュー表示
* バックエンドAPIによるファイル管理
* TauriによるデスクトップGUI
* 全文検索機能（キーワードによるドキュメント内容の検索）
* ファイル履歴の表示（Git統合機能）
* 特定バージョンのドキュメント表示
* メタデータ管理（SQLite）
  * タグ機能
  * 最近更新されたドキュメント表示
  * ドキュメントのメタデータ編集
* ユーザー認証・認可システム
  * ユーザー登録・ログイン機能
  * JWT認証
  * ロールベースのアクセス制御（管理者/編集者/閲覧者）
  * パスワードハッシュ化（Argon2）

## 今後実装予定の機能

* ~~ファイル履歴の表示（Git統合の強化）~~ ✅
* ~~SQLiteによるメタデータ管理~~ ✅
* ~~ユーザー認証・認可システム~~ ✅
* 編集権限設定（オプション）
* スクリプトによるデプロイ・セットアップ自動化

## セットアップ手順

### 1. Rust環境構築

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Node.js環境構築

最新のNode.jsとnpmをインストールしてください。

### 3. プロジェクトのクローン

```bash
git clone https://github.com/yourusername/md-wiki-rust.git
cd md-wiki-rust
```

### 4. バックエンドの依存関係インストール

```bash
cd backend
cargo build
```

### 5. フロントエンドの依存関係インストール

```bash
cd frontend
npm install
```

### 6. Markdownファイル用のGitリポジトリ初期化

```bash
mkdir -p storage/markdown_files
cd storage/markdown_files
git init
```

## 実行方法

### バックエンド起動

#### 通常の起動

```bash
cd backend
cargo run
```

#### Dockerを使用した起動

1.  **.envファイルの作成**:
    リポジトリのルートに`.env`ファイルを作成し、環境変数を設定します。

    ```
    DATABASE_URL=sqlite:/app/storage/metadata.db
    JWT_SECRET=your_jwt_secret
    ```

2.  **Dockerコンテナのビルドと起動**:

    ```bash
    docker-compose up --build -d
    ```

3.  **コンテナの停止**:

    ```bash
    docker-compose down
    ```

4.  **コンテナへのアクセス**:
    実行中のコンテナのシェルにアクセスするには、以下のコマンドを実行します。

    ```bash
    docker-compose exec app /bin/bash
    ```

### フロントエンド起動（Tauri）

```bash
cd frontend
npm run tauri dev
```

## ライセンス

MIT License

## 今後の拡張性

* リアルタイム共同編集機能の導入
* 公開・非公開モードの切り替え
