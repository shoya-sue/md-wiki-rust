import React, { useState, useEffect } from 'react';
import { Link, useSearchParams } from 'react-router-dom';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

function SearchResults() {
  const [searchParams] = useSearchParams();
  const query = searchParams.get('q') || '';
  
  const [results, setResults] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [totalMatches, setTotalMatches] = useState(0);

  useEffect(() => {
    if (query) {
      fetchSearchResults();
    }
  }, [query]);

  const fetchSearchResults = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await fetch(`${API_BASE_URL}/api/documents/search?q=${encodeURIComponent(query)}`);
      
      if (!response.ok) {
        throw new Error(`検索に失敗しました: ${response.status}`);
      }
      
      const data = await response.json();
      setResults(data.results);
      setTotalMatches(data.total_matches);
    } catch (err) {
      setError(err.message);
      console.error('検索エラー:', err);
    } finally {
      setLoading(false);
    }
  };

  // 検索結果中のキーワードをハイライトする関数
  const highlightMatches = (text) => {
    if (!query || !text) return text;
    
    const parts = text.split(new RegExp(`(${query})`, 'gi'));
    return parts.map((part, i) => 
      part.toLowerCase() === query.toLowerCase() 
        ? <mark key={i}>{part}</mark> 
        : part
    );
  };

  if (!query) {
    return <div className="search-results-empty">検索キーワードを入力してください</div>;
  }

  if (loading) {
    return <div className="loading">検索中...</div>;
  }

  if (error) {
    return <div className="error-message">エラー: {error}</div>;
  }

  return (
    <div className="search-results">
      <h2>「{query}」の検索結果</h2>
      
      {results.length === 0 ? (
        <p>検索結果が見つかりませんでした。別のキーワードで試してください。</p>
      ) : (
        <>
          <p className="search-summary">
            {results.length}個のドキュメントで合計{totalMatches}件の一致が見つかりました
          </p>
          
          <ul className="search-results-list">
            {results.map((result) => (
              <li key={result.filename} className="search-result-item">
                <h3>
                  <Link to={`/view/${result.filename}`}>{result.filename}</Link>
                </h3>
                <div className="search-preview">
                  {highlightMatches(result.content_preview)}
                </div>
                <div className="search-meta">
                  一致数: {result.matches} | 
                  <Link to={`/edit/${result.filename}`} className="edit-link">
                    編集
                  </Link>
                </div>
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  );
}

export default SearchResults; 