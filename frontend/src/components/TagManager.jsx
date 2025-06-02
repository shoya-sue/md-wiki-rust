import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import '../styles.css';

function TagManager() {
  const [tags, setTags] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [selectedTag, setSelectedTag] = useState(null);
  const [documents, setDocuments] = useState([]);
  const [documentsLoading, setDocumentsLoading] = useState(false);

  // すべてのタグを取得
  useEffect(() => {
    const fetchTags = async () => {
      try {
        setLoading(true);
        const response = await fetch('http://localhost:3000/api/tags');
        
        if (!response.ok) {
          throw new Error(`Error: ${response.status}`);
        }
        
        const data = await response.json();
        setTags(data.tags || []);
        setLoading(false);
      } catch (err) {
        setError(`タグの取得に失敗しました: ${err.message}`);
        setLoading(false);
      }
    };

    fetchTags();
  }, []);

  // タグが選択されたときにドキュメントを取得
  useEffect(() => {
    if (!selectedTag) {
      setDocuments([]);
      return;
    }

    const fetchDocumentsByTag = async () => {
      try {
        setDocumentsLoading(true);
        const response = await fetch(`http://localhost:3000/api/tags/search?tag=${encodeURIComponent(selectedTag)}`);
        
        if (!response.ok) {
          throw new Error(`Error: ${response.status}`);
        }
        
        const data = await response.json();
        setDocuments(data.documents || []);
        setDocumentsLoading(false);
      } catch (err) {
        setError(`ドキュメントの取得に失敗しました: ${err.message}`);
        setDocumentsLoading(false);
      }
    };

    fetchDocumentsByTag();
  }, [selectedTag]);

  // タグをクリックしたときの処理
  const handleTagClick = (tag) => {
    setSelectedTag(tag);
  };

  // 日付をフォーマット
  const formatDate = (timestamp) => {
    if (!timestamp) return '';
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
  };

  if (loading) {
    return <div className="loading">タグを読み込み中...</div>;
  }

  if (error) {
    return <div className="error-message">{error}</div>;
  }

  return (
    <div className="tag-manager">
      <div className="tag-manager-header">
        <h2>タグ一覧</h2>
      </div>
      
      <div className="tag-manager-content">
        <div className="tag-list">
          <h3>利用可能なタグ</h3>
          {tags.length === 0 ? (
            <p>タグがありません。</p>
          ) : (
            <div className="tags">
              {tags.map((tag) => (
                <span 
                  key={tag} 
                  className={`tag ${selectedTag === tag ? 'tag-selected' : ''}`}
                  onClick={() => handleTagClick(tag)}
                >
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>
        
        {selectedTag && (
          <div className="tagged-documents">
            <h3>タグ「{selectedTag}」のドキュメント</h3>
            
            {documentsLoading ? (
              <div className="loading">ドキュメントを読み込み中...</div>
            ) : documents.length === 0 ? (
              <p>このタグのドキュメントはありません。</p>
            ) : (
              <ul className="documents">
                {documents.map((doc) => (
                  <li key={doc.id} className="document-item">
                    <div className="document-info">
                      <span className="document-name">{doc.title || doc.filename}</span>
                      <div className="document-meta">
                        <span className="document-date">更新日: {formatDate(doc.updated_at)}</span>
                        <span className="document-views">閲覧数: {doc.view_count}</span>
                      </div>
                      <div className="document-tags">
                        {doc.tags.map((tag) => (
                          <span key={tag} className="tag">{tag}</span>
                        ))}
                      </div>
                    </div>
                    <div className="document-actions">
                      <Link to={`/view/${doc.filename}`} className="view-btn">表示</Link>
                      <Link to={`/edit/${doc.filename}`} className="edit-btn">編集</Link>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default TagManager; 