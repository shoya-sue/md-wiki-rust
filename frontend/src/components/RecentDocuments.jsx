import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import '../styles.css';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

function RecentDocuments({ limit = 5 }) {
  const [documents, setDocuments] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchRecentDocuments = async () => {
      try {
        setLoading(true);
        const response = await fetch(`${API_BASE_URL}/api/recent?limit=${limit}`);
        
        if (!response.ok) {
          throw new Error(`Error: ${response.status}`);
        }
        
        const data = await response.json();
        setDocuments(data.documents || []);
        setLoading(false);
      } catch (err) {
        setError(`最近のドキュメントの取得に失敗しました: ${err.message}`);
        setLoading(false);
      }
    };

    fetchRecentDocuments();
  }, [limit]);

  // 日付をフォーマット
  const formatDate = (timestamp) => {
    if (!timestamp) return '';
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
  };

  if (loading) {
    return <div className="loading">最近のドキュメントを読み込み中...</div>;
  }

  if (error) {
    return <div className="error-message">{error}</div>;
  }

  return (
    <div className="recent-documents">
      <div className="recent-documents-header">
        <h2>最近更新されたドキュメント</h2>
      </div>
      
      {documents.length === 0 ? (
        <p>最近更新されたドキュメントはありません。</p>
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
                {doc.tags && doc.tags.length > 0 && (
                  <div className="document-tags">
                    {doc.tags.map((tag) => (
                      <span key={tag} className="tag">{tag}</span>
                    ))}
                  </div>
                )}
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
  );
}

export default RecentDocuments; 