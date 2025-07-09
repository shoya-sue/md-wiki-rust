# API仕様書

このドキュメントでは、md-wiki-rustのバックエンドAPIの仕様について説明します。

## 基本情報

- ベースURL: `http://localhost:3000`
- データ形式: JSON
- 認証: 現在は実装されていません

## エンドポイント

### ヘルスチェック

```
GET /api/health
```

サーバーの状態を確認するためのエンドポイントです。

#### レスポンス

**成功時 (200 OK)**

```json
{
  "status": "ok"
}
```



**成功時 (200 OK)**

```json
{
  "id": 1,
  "filename": "welcome",
  "title": "ウェルカムページ",
  "created_at": 1633046400,
  "updated_at": 1633132800,
  "view_count": 42,
  "tags": ["タグ1", "タグ2", "タグ3"]
}
```

**エラー時 (404 Not Found)**

```json
{
  "error": "Metadata for document welcome not found"
}
```

### すべてのタグを取得

```
GET /api/tags
```

システム内のすべてのタグを取得します。

#### レスポンス

**成功時 (200 OK)**

```json
{
  "tags": ["タグ1", "タグ2", "タグ3", "タグ4"]
}
```

### タグによるドキュメント検索

```
GET /api/tags/search?tag=タグ名
```

特定のタグが付いたドキュメントを検索します。

#### パラメータ

- `tag`: 検索するタグ名

#### レスポンス

**成功時 (200 OK)**

```json
{
  "documents": [
    {
      "id": 1,
      "filename": "welcome",
      "title": "ウェルカムページ",
      "created_at": 1633046400,
      "updated_at": 1633132800,
      "view_count": 42,
      "tags": ["タグ1", "タグ2"]
    },
    {
      "id": 2,
      "filename": "about",
      "title": "このサイトについて",
      "created_at": 1633046500,
      "updated_at": 1633132900,
      "view_count": 28,
      "tags": ["タグ1", "タグ3"]
    }
  ]
}
```

**エラー時 (400 Bad Request)**

```json
{
  "error": "Tag parameter is required"
}
```

### 最近更新されたドキュメント

```
GET /api/recent?limit=5
```

最近更新されたドキュメントのリストを取得します。

#### パラメータ

- `limit`: 取得するドキュメント数（デフォルト: 10）

#### レスポンス

**成功時 (200 OK)**

```json
{
  "documents": [
    {
      "id": 1,
      "filename": "welcome",
      "title": "ウェルカムページ",
      "created_at": 1633046400,
      "updated_at": 1633132800,
      "view_count": 42,
      "tags": ["タグ1", "タグ2"]
    },
    // 他のドキュメント...
  ]
}
```

## 今後実装予定のエンドポイント

### ドキュメント履歴の取得

```
GET /api/documents/:filename/history
```

特定のドキュメントの変更履歴を取得します。

#### パラメータ

- `filename`: 履歴を取得するドキュメントのファイル名（拡張子なし）

#### レスポンス

**成功時 (200 OK)**

```json
{
  "filename": "welcome",
  "commits": [
    {
      "id": "8a7d6e5f4c3b2a1098765432100abcdef1234567",
      "message": "Update welcome.md",
      "author": "MD Wiki User",
      "email": "user@example.com",
      "time": 1633046400,
      "timestamp": "2021-10-01 12:00:00"
    },
    {
      "id": "1234567890abcdef1234567890abcdef12345678",
      "message": "Initial commit",
      "author": "MD Wiki User",
      "email": "user@example.com",
      "time": 1632960000,
      "timestamp": "2021-09-30 12:00:00"
    }
  ]
}
```

**エラー時 (500 Internal Server Error)**

```json
{
  "error": "Failed to get document history: [エラーメッセージ]"
}
```

### 特定バージョンのドキュメント取得

```
GET /api/documents/:filename/version/:commit_id
```

特定のバージョン（コミットID）のドキュメントを取得します。

#### パラメータ

- `filename`: 取得するドキュメントのファイル名（拡張子なし）
- `commit_id`: 取得するコミットのID（省略可能な接頭辞）

#### レスポンス

**成功時 (200 OK)**

```json
{
  "filename": "welcome",
  "content": "# Welcome\n\nThis is an old version of the document.",
  "commit_info": {
    "id": "8a7d6e5f4c3b2a1098765432100abcdef1234567",
    "message": "Update welcome.md",
    "author": "MD Wiki User",
    "email": "user@example.com",
    "time": 1633046400,
    "timestamp": "2021-10-01 12:00:00"
  }
}
```

**エラー時 (404 Not Found)**

```json
{
  "error": "Commit 8a7d6e5 not found for document welcome"
}
```

**エラー時 (500 Internal Server Error)**

```json
{
  "error": "Failed to get document at commit 8a7d6e5: [エラーメッセージ]"
}
```

### メタデータの取得と更新

```
```

## 認証API

### POST /api/auth/register
新規ユーザー登録

**リクエスト**
```json
{
  "username": "string",
  "password": "string",
  "role": "string" // "admin", "editor", "viewer"のいずれか
}
```

**レスポンス**
```json
{
  "token": "string", // JWTトークン
  "user": {
    "id": "number",
    "username": "string",
    "role": "string"
  }
}
```

### POST /api/auth/login
ログイン

**リクエスト**
```json
{
  "username": "string",
  "password": "string"
}
```

**レスポンス**
```json
{
  "token": "string", // JWTトークン
  "user": {
    "id": "number",
    "username": "string",
    "role": "string"
  }
}
```

## ドキュメントAPI

### GET /api/documents
ドキュメント一覧取得

**レスポンス**
```json
{
  "documents": [
    {
      "id": "number",
      "filename": "string",
      "title": "string",
      "created_at": "string",
      "updated_at": "string",
      "tags": ["string"]
    }
  ]
}
```

### GET /api/documents/{filename}
ドキュメント取得

**レスポンス**
```json
{
  "content": "string",
  "metadata": {
    "id": "number",
    "filename": "string",
    "title": "string",
    "created_at": "string",
    "updated_at": "string",
    "tags": ["string"]
  }
}
```

### POST /api/documents
ドキュメント作成

**リクエスト**
```json
{
  "filename": "string",
  "content": "string",
  "title": "string",
  "tags": ["string"]
}
```

### PUT /api/documents/{filename}
ドキュメント更新

**リクエスト**
```json
{
  "content": "string",
  "title": "string",
  "tags": ["string"]
}
```

### DELETE /api/documents/{filename}
ドキュメント削除

## タグAPI

### GET /api/tags
タグ一覧取得

**レスポンス**
```json
{
  "tags": [
    {
      "id": "number",
      "name": "string"
    }
  ]
}
```

### GET /api/tags/{name}/documents
タグに関連付けられたドキュメント一覧取得

**レスポンス**
```json
{
  "documents": [
    {
      "id": "number",
      "filename": "string",
      "title": "string",
      "created_at": "string",
      "updated_at": "string",
      "tags": ["string"]
    }
  ]
}
```

## Git操作API

### GET /api/git/history/{filename}
ドキュメントの変更履歴取得

**レスポンス**
```json
{
  "history": [
    {
      "commit_id": "string",
      "author": "string",
      "date": "string",
      "message": "string"
    }
  ]
}
```

### GET /api/git/version/{filename}/{commit_id}
特定バージョンのドキュメント取得

**レスポンス**
```json
{
  "content": "string",
  "commit": {
    "id": "string",
    "author": "string",
    "date": "string",
    "message": "string"
  }
}
```