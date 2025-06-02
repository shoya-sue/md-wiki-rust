# セットアップ手順

このドキュメントでは、md-wiki-rustプロジェクトの開発環境のセットアップ方法と実行方法について説明します。

## 前提条件

以下のソフトウェアがインストールされていることを確認してください：

- Rust (rustc 1.74.0以上)
- Node.js (v20.0.0以上)
- npm (9.0.0以上)
- Git

## 開発環境のセットアップ

### 1. リポジトリのクローン

```bash
git clone https://github.com/yourusername/md-wiki-rust.git
cd md-wiki-rust
```

### 2. バックエンドのセットアップ

```bash
cd backend
cargo build
```

### 3. フロントエンドのセットアップ

```bash
cd frontend
npm install
```

### 4. Markdownファイル用のGitリポジトリの初期化

```bash
mkdir -p storage/markdown_files
cd storage/markdown_files
git init
```

## アプリケーションの実行

### バックエンドの実行

```bash
cd backend
cargo run
```

これにより、APIサーバーがポート3000で起動します。

### フロントエンドの実行

別のターミナルで以下のコマンドを実行します：

```bash
cd frontend
npm run tauri dev
```

これにより、Tauriアプリケーションが開発モードで起動します。

## ビルド方法

### バックエンドのビルド

```bash
cd backend
cargo build --release
```

### フロントエンドのビルド

```bash
cd frontend
npm run tauri build
```

ビルドされたアプリケーションは `frontend/src-tauri/target/release` ディレクトリに作成されます。

## トラブルシューティング

### バックエンドの起動に失敗する場合

- ポート3000が他のアプリケーションで使用されていないか確認してください。
- ファイアウォールの設定を確認してください。

### フロントエンドのビルドに失敗する場合

- Node.jsとnpmのバージョンを確認してください。
- 依存関係が正しくインストールされているか確認してください。

```bash
cd frontend
npm ci
```

### Tauriのビルドに失敗する場合

- Rustのバージョンを確認してください。
- OSに必要な開発ツールがインストールされているか確認してください。
  - Windows: Visual Studio BuildTools
  - macOS: Xcode Command Line Tools
  - Linux: 必要なライブラリ（libwebkit2gtk-4.0-dev など）

## 開発リソース

- [Rust公式ドキュメント](https://www.rust-lang.org/ja/learn)
- [Axum公式ドキュメント](https://docs.rs/axum)
- [Tauri公式ドキュメント](https://tauri.app/v1/guides)
- [React公式ドキュメント](https://reactjs.org/docs/getting-started.html) 