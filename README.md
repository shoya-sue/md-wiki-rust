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
  * ReactまたはVueをフロントエンドフレームワークとして使用可能。

### バックエンド（Rust API）

* **Axum**

  * シンプルで直感的、高速で非同期処理対応のRustフレームワーク。
* **pulldown-cmark**

  * Markdownを解析してHTMLに変換するためのライブラリ。

### データストア

* Markdownファイルのバージョン管理はGitを推奨。
* SQLiteを使用してメタデータ管理。

## 推奨リポジトリ構造

```
md-wiki-rust/
├── frontend
│   ├── src
│   └── public
├── backend
│   ├── src
│   │   ├── routes
│   │   ├── handlers
│   │   └── main.rs
│   └── Cargo.toml
├── storage
│   └── markdown_files
├── scripts
│   ├── deploy.sh
│   └── setup.sh
├── README.md
└── LICENSE
```

## 主な機能

* Markdownファイル作成・編集
* ファイル履歴の表示（Git統合）
* Markdownプレビュー
* 検索機能
* 編集権限設定（オプション）

## セットアップ手順

### 1. Rust環境構築

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Tauriのセットアップ

```bash
npm create tauri-app@latest
# ReactまたはVueを選択
```

### 3. バックエンド構築（Axum）

```bash
cargo new backend
cd backend
cargo add axum tokio serde serde_json pulldown-cmark
```

### 4. データストア初期化（SQLite）

```bash
cargo add rusqlite
```

### 5. Gitリポジトリ初期化

```bash
mkdir storage/markdown_files
cd storage/markdown_files
git init
```

## 実行方法

### バックエンド起動

```bash
cd backend
cargo run
```

### フロントエンド起動（Tauri）

```bash
cd frontend
npm run tauri dev
```

## ライセンス

MIT License

## 今後の拡張性

* ユーザー認証・認可システム追加
* リアルタイム共同編集機能の導入
* 公開・非公開モードの切り替え

