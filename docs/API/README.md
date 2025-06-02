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

### ドキュメント一覧の取得

```
GET /api/wiki
```

利用可能なすべてのドキュメントの一覧を取得します。

#### レスポンス

**成功時 (200 OK)**

```json
{
  "documents": [
    "welcome",
    "getting-started",
    "example"
  ]
}
```

**エラー時 (500 Internal Server Error)**

```json
{
  "error": "Failed to read markdown directory: [エラーメッセージ]"
}
```

### ドキュメントの取得

```
GET /api/wiki/:filename
```

指定されたファイル名のドキュメントを取得します。

#### パラメータ

- `filename`: 取得するドキュメントのファイル名（拡張子なし）

#### レスポンス

**成功時 (200 OK)**

```json
{
  "filename": "welcome",
  "content": "# Welcome\n\nThis is a welcome document."
}
```

**エラー時 (404 Not Found)**

```json
{
  "error": "Document welcome not found"
}
```

### ドキュメントの保存

```
POST /api/wiki/:filename
```

指定されたファイル名でドキュメントを保存します。

#### パラメータ

- `filename`: 保存するドキュメントのファイル名（拡張子なし）

#### リクエスト本文

```json
{
  "filename": "welcome",
  "content": "# Welcome\n\nThis is an updated welcome document."
}
```

#### レスポンス

**成功時 (200 OK)**

ステータスコード200のみが返されます。

**エラー時 (500 Internal Server Error)**

```json
{
  "error": "Failed to save document: [エラーメッセージ]"
}
```

## 今後実装予定のエンドポイント

### ドキュメント検索

```
GET /api/wiki/search?q=検索クエリ
```

ドキュメント内の内容を検索します。

### ドキュメント履歴の取得

```
GET /api/wiki/:filename/history
```

特定のドキュメントの変更履歴を取得します。

### 特定バージョンのドキュメント取得

```
GET /api/wiki/:filename/version/:commit_id
```

特定のバージョン（コミットID）のドキュメントを取得します。

### メタデータの取得と更新

```
GET /api/wiki/:filename/metadata
POST /api/wiki/:filename/metadata
```

ドキュメントのメタデータ（タグ、作成日時など）を取得・更新します。 