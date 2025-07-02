use std::fmt;
use git2::{Repository, Signature, Time, Commit, Oid, ObjectType, Error as GitError};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Serialize, Deserialize};
use chrono::DateTime;
use crate::error::AppError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub author: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Clone)]
pub struct GitRepository {
    repo: Arc<Mutex<Repository>>,
    root_path: PathBuf,
}

impl std::fmt::Debug for GitRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitRepository")
            .field("root_path", &self.root_path)
            .finish()
    }
}

impl GitRepository {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, GitError> {
        let root_path = path.as_ref().to_path_buf();
        let repo = Repository::init(&root_path)?;
        Ok(Self {
            repo: Arc::new(Mutex::new(repo)),
            root_path,
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, GitError> {
        let root_path = path.as_ref().to_path_buf();
        let repo = Repository::open(&root_path)?;
        Ok(Self {
            repo: Arc::new(Mutex::new(repo)),
            root_path,
        })
    }

    pub fn commit_file<P: AsRef<Path>>(&self, file_path: P, message: &str, author: &str) -> Result<Oid, GitError> {
        let path = file_path.as_ref();
        let repo = self.repo.lock();
        let mut index = repo.index()?;
        
        index.add_path(path)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let signature = Signature::now(author, "user@example.com")?;
        let parent_commit = repo.head().ok().and_then(|head| head.target()).and_then(|oid| repo.find_commit(oid).ok());
        
        let parents = match parent_commit {
            Some(ref commit) => vec![commit],
            None => vec![],
        };

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )
    }

    pub fn get_file_history<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<CommitInfo>, GitError> {
        let path = file_path.as_ref();
        let repo = self.repo.lock();
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        
        let mut history = Vec::new();
        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            
            if let Some(parent) = commit.parent(0).ok() {
                let diff = repo.diff_tree_to_tree(
                    Some(&parent.tree()?),
                    Some(&commit.tree()?),
                    None,
                )?;

                let mut found = false;
                diff.foreach(
                    &mut |delta, _| {
                        if let Some(file) = delta.new_file().path() {
                            if file == path {
                                found = true;
                                return false; // Stop iteration
                            }
                        }
                        true
                    },
                    None,
                    None,
                    None,
                )?;

                if found {
                    history.push(CommitInfo {
                        id: commit.id().to_string(),
                        author: commit.author().name().unwrap_or("Unknown").to_string(),
                        message: commit.message().unwrap_or("").to_string(),
                        timestamp: commit.time().seconds(),
                    });
                }
            }
        }

        Ok(history)
    }

    pub fn get_file_content_at_commit<P: AsRef<Path>>(&self, file_path: P, commit_id: &str) -> Result<String, GitError> {
        let path = file_path.as_ref();
        let repo = self.repo.lock();
        let oid = Oid::from_str(commit_id)?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        
        if let Ok(entry) = tree.get_path(path) {
            let blob = repo.find_blob(entry.id())?;
            String::from_utf8(blob.content().to_vec())
                .map_err(|_| GitError::from_str("Invalid UTF-8 content"))
        } else {
            Err(GitError::from_str("File not found in commit"))
        }
    }

    // ファイルの変更履歴を取得
    pub fn get_file_changes(&self, file_path: &str) -> Result<Vec<CommitInfo>, GitError> {
        let full_path = self.root_path.join(file_path);
        let relative_path = full_path.strip_prefix(&self.root_path)
            .unwrap_or_else(|_| Path::new(file_path));
        
        let repo = self.repo.lock();
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        
        let mut commits = Vec::new();
        
        for oid in revwalk {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            
            // このコミットでファイルが変更されたかチェック
            if self.file_changed_in_commit(&commit, relative_path)? {
                commits.push(self.commit_to_info(&commit)?);
            }
        }
        
        Ok(commits)
    }
    
    // 特定のコミットでファイルが変更されたかチェック
    fn file_changed_in_commit(&self, commit: &Commit, file_path: &Path) -> Result<bool, GitError> {
        if commit.parent_count() == 0 {
            // 最初のコミットの場合、ツリーからファイルを探す
            let tree = commit.tree()?;
            return Ok(tree.get_path(file_path).is_ok());
        }
        
        let parent = commit.parent(0)?;
        let parent_tree = parent.tree()?;
        let commit_tree = commit.tree()?;
        
        let repo = self.repo.lock();
        let diff = repo.diff_tree_to_tree(
            Some(&parent_tree),
            Some(&commit_tree),
            None,
        )?;
        
        let mut found = false;
        diff.foreach(
            &mut |_delta, _progress| { true },
            None,
            None,
            Some(&mut |diff_file, _binary, _| {
                if let Some(diff_path) = diff_file.new_file().path() {
                    if diff_path == file_path {
                        found = true;
                        return false;
                    }
                }
                true
            }),
        )?;
        
        Ok(found)
    }
    
    // コミットオブジェクトをCommitInfo構造体に変換
    fn commit_to_info(&self, commit: &Commit) -> Result<CommitInfo, GitError> {
        let author = commit.author();
        let time = author.when();
        
        Ok(CommitInfo {
            id: commit.id().to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: author.name().unwrap_or("Unknown").to_string(),
            timestamp: time.seconds(),
        })
    }
    
    // 署名（コミット作者情報）を作成
    fn get_signature(&self) -> Result<Signature<'_>, GitError> {
        let repo = self.repo.lock();
        let config = repo.config()?;
        
        let name = config.get_string("user.name")
            .unwrap_or_else(|_| "MD Wiki User".to_string());
            
        let email = config.get_string("user.email")
            .unwrap_or_else(|_| "user@md-wiki.example".to_string());
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
            
        Ok(Signature::new(&name, &email, &Time::new(now, 0))?)
    }
    
    // HEADコミットを取得
    fn get_head_commit(repo: &Repository) -> Result<Commit, GitError> {
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        Ok(head_commit)
    }
    
    // ファイルを削除してコミット
    pub fn remove_file(&self, file_path: &str, message: &str) -> Result<String, GitError> {
        let full_path = self.root_path.join(file_path);
        let relative_path = full_path.strip_prefix(&self.root_path)
            .unwrap_or_else(|_| Path::new(file_path));
        
        // ファイルが存在するか確認
        if !full_path.exists() {
            return Err(GitError::from_str(&format!("File {} does not exist", file_path)));
        }
        
        let repo = self.repo.lock();
        
        // インデックスからファイルを削除
        let mut index = repo.index()?;
        index.remove_path(relative_path)?;
        index.write()?;
        
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        // コミット作成
        let signature = self.get_signature()?;
        let parent_commit = GitRepository::get_head_commit(&repo)?;
        
        let commit_id = repo.commit(
            Some("HEAD"), 
            &signature, 
            &signature, 
            message, 
            &tree, 
            &[&parent_commit],
        )?;
        
        Ok(commit_id.to_string())
    }
    
    // リポジトリの全コミット履歴を取得
    pub fn get_repo_history(&self, limit: usize) -> Result<Vec<CommitInfo>, GitError> {
        let repo = self.repo.lock();
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        
        let mut commits = Vec::new();
        
        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }
            
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            commits.push(self.commit_to_info(&commit)?);
        }
        
        Ok(commits)
    }
}

// Unixタイムスタンプをフォーマットする関数
fn format_timestamp(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

impl CommitInfo {
    pub fn from_commit(commit: &Commit) -> Self {
        let author = commit.author();
        let time = commit.time();
        
        CommitInfo {
            id: commit.id().to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: author.name().unwrap_or("").to_string(),
            timestamp: time.seconds(),
        }
    }
}

fn format_time(time: &Time) -> String {
    let dt = chrono::DateTime::from_timestamp(time.seconds(), 0)
        .unwrap_or(chrono::DateTime::from_timestamp(0, 0).unwrap());
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
} 