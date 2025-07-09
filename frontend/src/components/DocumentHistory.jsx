import React, { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import '../styles.css';

function DocumentHistory() {
  const { filename } = useParams();
  const [history, setHistory] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    const fetchHistory = async () => {
      try {
        setLoading(true);
        const response = await fetch(`http://localhost:3000/api/documents/${filename}/history`);
        
        if (!response.ok) {
          throw new Error(`Error: ${response.status}`);
        }
        
        const data = await response.json();
        setHistory(data.commits);
        setLoading(false);
      } catch (err) {
        setError(`履歴の取得に失敗しました: ${err.message}`);
        setLoading(false);
      }
    };

    fetchHistory();
  }, [filename]);

  if (loading) {
    return <div className="loading">履歴を読み込み中...</div>;
  }

  if (error) {
    return <div className="error-message">{error}</div>;
  }

  return (
    <div className="document-history">
      <div className="history-header">
        <h2>{filename} の変更履歴</h2>
        <Link to={`/view/${filename}`} className="view-btn">ドキュメントへ戻る</Link>
      </div>
      
      {history.length === 0 ? (
        <p>履歴がありません。</p>
      ) : (
        <ul className="history-list">
          {history.map((commit, index) => (
            <li key={commit.id} className="history-item">
              <div className="commit-info">
                <div className="commit-header">
                  <span className="commit-date">{commit.timestamp}</span>
                  <span className="commit-author">{commit.author} &lt;{commit.email}&gt;</span>
                </div>
                <p className="commit-message">{commit.message}</p>
                <div className="commit-actions">
                  <Link to={`/view/${filename}/version/${commit.id.substring(0, 7)}`} className="view-version-btn">
                    このバージョンを表示
                  </Link>
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default DocumentHistory; 