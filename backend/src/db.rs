use rusqlite::{Connection, Result as SqlResult, params};
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::models::{User, UserRole, hash_password, verify_password};

/// ドキュメントのメタデータを表す構造体
#[derive(Debug, Clone)]
pub struct DocumentMeta {
    pub id: i64,
    pub filename: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub view_count: i64,
    pub tags: Vec<String>,
}

/// データベース接続を管理する構造体
#[derive(Clone)]
pub struct DbManager {
    conn: Arc<Mutex<Connection>>,
}

impl DbManager {
    /// 新しいDBマネージャーを作成し、データベースを初期化する
    pub fn new(db_path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        
        // データベースを初期化
        Self::init_db(&conn)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    
    /// ユーザーテーブルの初期化
    fn init_user_tables(conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                role TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_login INTEGER
            )",
            [],
        )?;
        
        Ok(())
    }
    
    /// データベーススキーマを初期化
    fn init_db(conn: &Connection) -> SqlResult<()> {
        // 既存のテーブル初期化
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY,
                filename TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                view_count INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS document_tags (
                document_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (document_id, tag_id),
                FOREIGN KEY (document_id) REFERENCES documents (id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // ユーザーテーブルの初期化
        Self::init_user_tables(conn)?;
        
        // 管理者ユーザーが存在しない場合は作成（初期設定）
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE role = 'admin'")?;
        let admin_count: i64 = stmt.query_row([], |row| row.get(0))?;
        
        if admin_count == 0 {
            // デフォルトの管理者ユーザーを作成
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            
            let password_hash = hash_password("admin123").map_err(|e| {
                rusqlite::Error::UserFunctionError(Box::new(e))
            })?;
            
            conn.execute(
                "INSERT INTO users (username, password_hash, email, role, created_at) VALUES (?, ?, ?, ?, ?)",
                params![
                    "admin",
                    password_hash,
                    "admin@example.com",
                    "admin",
                    now
                ],
            )?;
        }
        
        Ok(())
    }
    
    /// ドキュメントのメタデータを保存
    pub fn save_document_meta(&self, meta: &DocumentMeta) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        
        // トランザクション開始
        let tx = conn.transaction()?;
        
        // ドキュメントが存在するか確認
        let mut stmt = tx.prepare("SELECT id FROM documents WHERE filename = ?")?;
        let existing_id: Option<i64> = stmt.query_row(params![&meta.filename], |row| {
            Ok(row.get(0)?)
        }).ok();
        
        let document_id = if let Some(id) = existing_id {
            // 既存のドキュメントを更新
            tx.execute(
                "UPDATE documents SET title = ?, updated_at = ? WHERE id = ?",
                params![&meta.title, meta.updated_at, id],
            )?;
            id
        } else {
            // 新規ドキュメントを作成
            tx.execute(
                "INSERT INTO documents (filename, title, created_at, updated_at, view_count) VALUES (?, ?, ?, ?, ?)",
                params![&meta.filename, &meta.title, meta.created_at, meta.updated_at, 0],
            )?;
            tx.last_insert_rowid()
        };
        
        // 既存のタグ関連付けを削除
        tx.execute(
            "DELETE FROM document_tags WHERE document_id = ?",
            params![document_id],
        )?;
        
        // 新しいタグを登録または取得して関連付け
        for tag_name in &meta.tags {
            // タグが存在するか確認し、なければ作成
            let tag_id = self.get_or_create_tag(&tx, tag_name)?;
            
            // ドキュメントとタグを関連付け
            tx.execute(
                "INSERT OR IGNORE INTO document_tags (document_id, tag_id) VALUES (?, ?)",
                params![document_id, tag_id],
            )?;
        }
        
        // トランザクションをコミット
        tx.commit()?;
        
        Ok(document_id)
    }
    
    /// タグを取得または作成する
    fn get_or_create_tag(&self, tx: &rusqlite::Transaction, tag_name: &str) -> SqlResult<i64> {
        // タグが存在するか確認
        let mut stmt = tx.prepare("SELECT id FROM tags WHERE name = ?")?;
        let tag_id: Option<i64> = stmt.query_row(params![tag_name], |row| {
            Ok(row.get(0)?)
        }).ok();
        
        if let Some(id) = tag_id {
            Ok(id)
        } else {
            // 新しいタグを作成
            tx.execute(
                "INSERT INTO tags (name) VALUES (?)",
                params![tag_name],
            )?;
            Ok(tx.last_insert_rowid())
        }
    }
    
    /// ドキュメントのメタデータを取得
    pub fn get_document_meta(&self, filename: &str) -> SqlResult<Option<DocumentMeta>> {
        let conn = self.conn.lock().unwrap();
        
        // ドキュメントのメタデータを取得
        let mut stmt = conn.prepare(
            "SELECT id, filename, title, created_at, updated_at, view_count FROM documents WHERE filename = ?"
        )?;
        
        let document = stmt.query_row(params![filename], |row| {
            Ok(DocumentMeta {
                id: row.get(0)?,
                filename: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                view_count: row.get(5)?,
                tags: Vec::new(), // タグは別途取得
            })
        }).ok();
        
        if let Some(mut document) = document {
            // ドキュメントのタグを取得
            let mut stmt = conn.prepare(
                "SELECT t.name FROM tags t
                JOIN document_tags dt ON t.id = dt.tag_id
                WHERE dt.document_id = ?"
            )?;
            
            let tag_iter = stmt.query_map(params![document.id], |row| {
                let tag_name: String = row.get(0)?;
                Ok(tag_name)
            })?;
            
            for tag_result in tag_iter {
                if let Ok(tag) = tag_result {
                    document.tags.push(tag);
                }
            }
            
            Ok(Some(document))
        } else {
            Ok(None)
        }
    }
    
    /// 閲覧回数をインクリメント
    pub fn increment_view_count(&self, filename: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE documents SET view_count = view_count + 1 WHERE filename = ?",
            params![filename],
        )?;
        
        Ok(())
    }
    
    /// タグでドキュメントを検索
    pub fn search_documents_by_tag(&self, tag: &str) -> SqlResult<Vec<DocumentMeta>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT d.id, d.filename, d.title, d.created_at, d.updated_at, d.view_count
            FROM documents d
            JOIN document_tags dt ON d.id = dt.document_id
            JOIN tags t ON dt.tag_id = t.id
            WHERE t.name = ?"
        )?;
        
        let doc_iter = stmt.query_map(params![tag], |row| {
            Ok(DocumentMeta {
                id: row.get(0)?,
                filename: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                view_count: row.get(5)?,
                tags: Vec::new(), // タグは別途取得
            })
        })?;
        
        let mut documents = Vec::new();
        for doc_result in doc_iter {
            if let Ok(mut doc) = doc_result {
                // 各ドキュメントのタグを取得
                self.load_document_tags(&mut doc)?;
                documents.push(doc);
            }
        }
        
        Ok(documents)
    }
    
    /// ドキュメントのタグを読み込む
    fn load_document_tags(&self, doc: &mut DocumentMeta) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT t.name FROM tags t
            JOIN document_tags dt ON t.id = dt.tag_id
            WHERE dt.document_id = ?"
        )?;
        
        let tag_iter = stmt.query_map(params![doc.id], |row| {
            let tag_name: String = row.get(0)?;
            Ok(tag_name)
        })?;
        
        doc.tags.clear();
        for tag_result in tag_iter {
            if let Ok(tag) = tag_result {
                doc.tags.push(tag);
            }
        }
        
        Ok(())
    }
    
    /// すべてのタグを取得
    pub fn get_all_tags(&self) -> SqlResult<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare("SELECT name FROM tags ORDER BY name")?;
        let tag_iter = stmt.query_map([], |row| {
            let tag_name: String = row.get(0)?;
            Ok(tag_name)
        })?;
        
        let mut tags = Vec::new();
        for tag_result in tag_iter {
            if let Ok(tag) = tag_result {
                tags.push(tag);
            }
        }
        
        Ok(tags)
    }
    
    /// 最近更新されたドキュメントを取得
    pub fn get_recent_documents(&self, limit: usize) -> SqlResult<Vec<DocumentMeta>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, filename, title, created_at, updated_at, view_count
            FROM documents
            ORDER BY updated_at DESC
            LIMIT ?"
        )?;
        
        let doc_iter = stmt.query_map(params![limit as i64], |row| {
            Ok(DocumentMeta {
                id: row.get(0)?,
                filename: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                view_count: row.get(5)?,
                tags: Vec::new(),
            })
        })?;
        
        let mut documents = Vec::new();
        for doc_result in doc_iter {
            if let Ok(mut doc) = doc_result {
                self.load_document_tags(&mut doc)?;
                documents.push(doc);
            }
        }
        
        Ok(documents)
    }
    
    // ユーザーをIDで取得
    pub fn get_user_by_id(&self, id: i64) -> SqlResult<Option<User>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, email, role, created_at, last_login FROM users WHERE id = ?"
        )?;
        
        let user = stmt.query_row(params![id], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                email: row.get(3)?,
                role: UserRole::from_str(&row.get::<_, String>(4)?),
                created_at: row.get(5)?,
                last_login: row.get(6)?,
            })
        }).optional()?;
        
        Ok(user)
    }
    
    // ユーザー名でユーザーを取得
    pub fn get_user_by_username(&self, username: &str) -> SqlResult<Option<User>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, email, role, created_at, last_login FROM users WHERE username = ?"
        )?;
        
        let user = stmt.query_row(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                email: row.get(3)?,
                role: UserRole::from_str(&row.get::<_, String>(4)?),
                created_at: row.get(5)?,
                last_login: row.get(6)?,
            })
        }).optional()?;
        
        Ok(user)
    }
    
    // ユーザーを作成
    pub fn create_user(&self, username: &str, password: &str, email: &str, role: &str) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        
        // ユーザー名またはメールアドレスが既に使用されているかチェック
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE username = ? OR email = ?")?;
        let count: i64 = stmt.query_row(params![username, email], |row| row.get(0))?;
        
        if count > 0 {
            return Err(rusqlite::Error::UserFunctionError(Box::new(
                std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Username or email already exists")
            )));
        }
        
        // パスワードをハッシュ化
        let password_hash = match hash_password(password) {
            Ok(hash) => hash,
            Err(e) => return Err(rusqlite::Error::UserFunctionError(Box::new(e))),
        };
        
        // 現在のUNIXタイムスタンプを取得
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        // ユーザーを作成
        conn.execute(
            "INSERT INTO users (username, password_hash, email, role, created_at) VALUES (?, ?, ?, ?, ?)",
            params![username, password_hash, email, role, now],
        )?;
        
        Ok(conn.last_insert_rowid())
    }
    
    // ユーザーを認証
    pub fn authenticate_user(&self, username: &str, password: &str) -> SqlResult<Option<User>> {
        let user = self.get_user_by_username(username)?;
        
        if let Some(user) = user {
            match verify_password(&user.password_hash, password) {
                Ok(is_valid) => {
                    if is_valid {
                        // 最終ログイン時間を更新
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        
                        let conn = self.conn.lock().unwrap();
                        conn.execute(
                            "UPDATE users SET last_login = ? WHERE id = ?",
                            params![now, user.id],
                        )?;
                        
                        let mut updated_user = user;
                        updated_user.last_login = Some(now);
                        
                        return Ok(Some(updated_user));
                    }
                },
                Err(_) => return Ok(None),
            }
        }
        
        Ok(None)
    }
    
    // ユーザーのパスワードを変更
    pub fn change_password(&self, user_id: i64, current_password: &str, new_password: &str) -> SqlResult<bool> {
        let user = self.get_user_by_id(user_id)?;
        
        if let Some(user) = user {
            match verify_password(&user.password_hash, current_password) {
                Ok(is_valid) => {
                    if is_valid {
                        // 新しいパスワードをハッシュ化
                        let new_password_hash = match hash_password(new_password) {
                            Ok(hash) => hash,
                            Err(e) => return Err(rusqlite::Error::UserFunctionError(Box::new(e))),
                        };
                        
                        // パスワードを更新
                        let conn = self.conn.lock().unwrap();
                        conn.execute(
                            "UPDATE users SET password_hash = ? WHERE id = ?",
                            params![new_password_hash, user_id],
                        )?;
                        
                        return Ok(true);
                    }
                },
                Err(_) => return Ok(false),
            }
        }
        
        Ok(false)
    }
    
    // すべてのユーザーを取得
    pub fn get_all_users(&self) -> SqlResult<Vec<User>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, email, role, created_at, last_login FROM users"
        )?;
        
        let user_iter = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                email: row.get(3)?,
                role: UserRole::from_str(&row.get::<_, String>(4)?),
                created_at: row.get(5)?,
                last_login: row.get(6)?,
            })
        })?;
        
        let mut users = Vec::new();
        for user_result in user_iter {
            if let Ok(user) = user_result {
                users.push(user);
            }
        }
        
        Ok(users)
    }
    
    // ユーザーの情報を更新
    pub fn update_user(&self, user_id: i64, email: Option<&str>, role: Option<&str>) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        
        let user = self.get_user_by_id(user_id)?;
        if user.is_none() {
            return Ok(false);
        }
        
        let mut updates = Vec::new();
        let mut params_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        
        if let Some(email) = email {
            updates.push("email = ?");
            params_values.push(Box::new(email.to_string()));
        }
        
        if let Some(role) = role {
            updates.push("role = ?");
            params_values.push(Box::new(role.to_string()));
        }
        
        if updates.is_empty() {
            return Ok(true); // 何も更新がなければ成功とみなす
        }
        
        let sql = format!(
            "UPDATE users SET {} WHERE id = ?",
            updates.join(", ")
        );
        
        let mut params: Vec<&dyn rusqlite::ToSql> = params_values.iter().map(|p| p.as_ref()).collect();
        params.push(&user_id);
        
        let result = conn.execute(&sql, rusqlite::params_from_iter(params))?;
        
        Ok(result > 0)
    }
    
    // ユーザーを削除
    pub fn delete_user(&self, user_id: i64) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        
        let result = conn.execute(
            "DELETE FROM users WHERE id = ?",
            params![user_id],
        )?;
        
        Ok(result > 0)
    }
    
    /// ドキュメントのメタデータを削除
    pub fn delete_document_metadata(&self, filename: &str) -> SqlResult<bool> {
        let conn = self.conn.lock().unwrap();
        
        // トランザクション開始
        let tx = conn.transaction()?;
        
        // ドキュメントのIDを取得
        let mut stmt = tx.prepare("SELECT id FROM documents WHERE filename = ?")?;
        let document_id: Option<i64> = stmt.query_row(params![filename], |row| {
            Ok(row.get(0)?)
        }).ok();
        
        if let Some(id) = document_id {
            // ドキュメントとタグの関連付けを削除
            tx.execute(
                "DELETE FROM document_tags WHERE document_id = ?",
                params![id],
            )?;
            
            // ドキュメントを削除
            tx.execute(
                "DELETE FROM documents WHERE id = ?",
                params![id],
            )?;
            
            // 不要になったタグを削除（どのドキュメントにも関連付けられていないタグ）
            tx.execute(
                "DELETE FROM tags WHERE id NOT IN (SELECT DISTINCT tag_id FROM document_tags)",
                [],
            )?;
            
            // トランザクションをコミット
            tx.commit()?;
            
            Ok(true)
        } else {
            // ドキュメントが見つからない場合は何もしない
            tx.commit()?;
            Ok(false)
        }
    }
} 