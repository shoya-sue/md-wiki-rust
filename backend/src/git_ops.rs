use git2::{Repository, Signature, Time, Commit, Oid, ObjectType, Error as GitError};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub timestamp: String,
}

#[derive(Debug)]
pub struct GitOps {
    repo_path: PathBuf,
    repo: Repository,
}

impl GitOps {
    pub fn new(repo_path: &Path) -> Result<Self, GitError> {
        let repo = Repository::open(repo_path)?;
        Ok(Self {
            repo_path: repo_path.to_path_buf(),
            repo,
        })
    }

    // ファイルの変更履歴を取得
    pub fn get_file_history(&self, file_path: &str) -> Result<Vec<CommitInfo>, GitError> {
        let full_path = self.repo_path.join(file_path);
        let relative_path = full_path.strip_prefix(&self.repo_path)
            .unwrap_or_else(|_| Path::new(file_path));
        
        // リポジトリのHEADを取得
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        
        let mut commits = Vec::new();
        
        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            
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
        
        let diff = self.repo.diff_tree_to_tree(
            Some(&parent_tree),
            Some(&commit_tree),
            None,
        )?;
        
        let mut found = false;
        diff.foreach(
            &mut |_delta, _progress| { true },
            None,
            None,
            Some(&mut |diff_file, _| {
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
    
    // 特定のコミットバージョンのファイル内容を取得
    pub fn get_file_at_commit(&self, file_path: &str, commit_id: &str) -> Result<String, GitError> {
        let oid = Oid::from_str(commit_id)?;
        let commit = self.repo.find_commit(oid)?;
        let tree = commit.tree()?;
        
        let relative_path = Path::new(file_path);
        let entry = tree.get_path(relative_path)?;
        let object = entry.to_object(&self.repo)?;
        
        if let Some(blob) = object.as_blob() {
            let content = String::from_utf8_lossy(blob.content());
            Ok(content.to_string())
        } else {
            Err(GitError::from_str("Not a valid blob"))
        }
    }
    
    // ファイルをコミット
    pub fn commit_file(&self, file_path: &str, content: &str, message: &str) -> Result<String, GitError> {
        let full_path = self.repo_path.join(file_path);
        
        // ファイルに内容を書き込む
        std::fs::write(&full_path, content)
            .map_err(|e| GitError::from_str(&format!("Failed to write file: {}", e)))?;
        
        // インデックスに追加
        let mut index = self.repo.index()?;
        let relative_path = full_path.strip_prefix(&self.repo_path)
            .unwrap_or_else(|_| Path::new(file_path));
            
        index.add_path(relative_path)?;
        index.write()?;
        
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        // コミット作成
        let signature = self.get_signature()?;
        let parent_commit = self.get_head_commit()?;
        
        let commit_id = self.repo.commit(
            Some("HEAD"), 
            &signature, 
            &signature, 
            message, 
            &tree, 
            &[&parent_commit]
        )?;
        
        Ok(commit_id.to_string())
    }
    
    // コミットオブジェクトをCommitInfo構造体に変換
    fn commit_to_info(&self, commit: &Commit) -> Result<CommitInfo, GitError> {
        let author = commit.author();
        let time = author.when();
        
        let timestamp = format_timestamp(time.seconds());
        
        Ok(CommitInfo {
            id: commit.id().to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: author.name().unwrap_or("Unknown").to_string(),
            email: author.email().unwrap_or("").to_string(),
            time: time.seconds(),
            timestamp,
        })
    }
    
    // 署名（コミット作者情報）を作成
    fn get_signature(&self) -> Result<Signature<'_>, GitError> {
        let config = self.repo.config()?;
        
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
    fn get_head_commit(&self) -> Result<Commit, GitError> {
        let head = self.repo.head()?;
        let head_commit = head.peel_to_commit()?;
        Ok(head_commit)
    }
    
    // リポジトリの全コミット履歴を取得
    pub fn get_repo_history(&self, limit: usize) -> Result<Vec<CommitInfo>, GitError> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        
        let mut commits = Vec::new();
        
        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }
            
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            commits.push(self.commit_to_info(&commit)?);
        }
        
        Ok(commits)
    }
}

// Unixタイムスタンプをフォーマットする関数
fn format_timestamp(timestamp: i64) -> String {
    let dt = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0)
        .unwrap_or_else(|| chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
} 