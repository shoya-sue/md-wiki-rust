import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import ReactMarkdown from 'react-markdown';
import '../styles.css';

function DocumentVersionViewer() {
  const { filename, commitId } = useParams();
  const [document, setDocument] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchDocumentVersion = async () => {
      try {
        setLoading(true);
        const response = await fetch(`http://localhost:3000/api/wiki/${filename}/version/${commitId}`);
        
        if (!response.ok) {
          throw new Error(`Error: ${response.status}`);
        }
        
        const data = await response.json();
        setDocument(data);
        setLoading(false);
      } catch (err) {
        setError(`ドキュメントの読み込みに失敗しました: ${err.message}`);
        setLoading(false);
      }
    };

    fetchDocumentVersion();
  }, [filename, commitId]);

  if (loading) {
    return <div className="loading">ドキュメントを読み込み中...</div>;
  }

  if (error) {
    return <div className="error-message">{error}</div>;
  }

  if (!document) {
    return <div className="error-message">ドキュメントが見つかりません。</div>;
  }

  return (
    <div className="document-viewer">
      <div className="viewer-header">
        <h2>{filename}</h2>
        <div className="viewer-actions">
          <Link to={`/history/${filename}`} className="history-btn">履歴を表示</Link>
          <Link to={`/view/${filename}`} className="view-btn">最新版へ</Link>
        </div>
      </div>
      
      <div className="version-info">
        <p>
          <strong>コミット:</strong> {document.commit_info.id.substring(0, 7)}
          <span className="version-date"> ({document.commit_info.timestamp})</span>
        </p>
        <p><strong>作者:</strong> {document.commit_info.author}</p>
        <p><strong>メッセージ:</strong> {document.commit_info.message}</p>
      </div>
      
      <div className="markdown-content">
        <ReactMarkdown>{document.content}</ReactMarkdown>
      </div>
    </div>
  );
}

export default DocumentVersionViewer; 