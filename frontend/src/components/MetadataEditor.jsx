import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import '../styles.css';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

function MetadataEditor() {
  const { filename } = useParams();
  const navigate = useNavigate();
  
  const [metadata, setMetadata] = useState({
    title: '',
    tags: []
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [success, setSuccess] = useState(false);
  const [newTag, setNewTag] = useState('');
  const [availableTags, setAvailableTags] = useState([]);

  // メタデータとタグを取得
  useEffect(() => {
    const fetchData = async () => {
      try {
        setLoading(true);
        
        // メタデータを取得
        const metaResponse = await fetch(`${API_BASE_URL}/api/documents/${filename}/metadata`);
        
        // タグ一覧を取得
        const tagsResponse = await fetch(`${API_BASE_URL}/api/tags`);
        
        if (!tagsResponse.ok) {
          throw new Error(`Failed to fetch tags: ${tagsResponse.status}`);
        }
        
        const tagsData = await tagsResponse.json();
        setAvailableTags(tagsData.tags || []);
        
        if (metaResponse.ok) {
          const metaData = await metaResponse.json();
          setMetadata({
            title: metaData.title || filename,
            tags: metaData.tags || []
          });
        } else if (metaResponse.status === 404) {
          // メタデータが存在しない場合はデフォルト値を設定
          setMetadata({
            title: filename,
            tags: []
          });
        } else {
          throw new Error(`Failed to fetch metadata: ${metaResponse.status}`);
        }
        
        setLoading(false);
      } catch (err) {
        setError(`データの取得に失敗しました: ${err.message}`);
        setLoading(false);
      }
    };

    fetchData();
  }, [filename]);

  // メタデータを保存
  const handleSubmit = async (e) => {
    e.preventDefault();
    
    try {
      const response = await fetch(`${API_BASE_URL}/api/documents/${filename}/metadata`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(metadata),
      });
      
      if (!response.ok) {
        throw new Error(`Error: ${response.status}`);
      }
      
      setSuccess(true);
      setTimeout(() => {
        setSuccess(false);
      }, 3000);
    } catch (err) {
      setError(`メタデータの保存に失敗しました: ${err.message}`);
    }
  };

  // タイトル変更処理
  const handleTitleChange = (e) => {
    setMetadata({
      ...metadata,
      title: e.target.value
    });
  };

  // タグを追加
  const handleAddTag = () => {
    if (newTag.trim() === '') return;
    if (metadata.tags.includes(newTag.trim())) return;
    
    setMetadata({
      ...metadata,
      tags: [...metadata.tags, newTag.trim()]
    });
    setNewTag('');
  };

  // タグを削除
  const handleRemoveTag = (tagToRemove) => {
    setMetadata({
      ...metadata,
      tags: metadata.tags.filter(tag => tag !== tagToRemove)
    });
  };

  // 既存のタグを選択
  const handleSelectTag = (tag) => {
    if (metadata.tags.includes(tag)) return;
    
    setMetadata({
      ...metadata,
      tags: [...metadata.tags, tag]
    });
  };

  if (loading) {
    return <div className="loading">メタデータを読み込み中...</div>;
  }

  if (error && !loading) {
    return <div className="error-message">{error}</div>;
  }

  return (
    <div className="metadata-editor">
      <div className="metadata-editor-header">
        <h2>{filename} のメタデータを編集</h2>
      </div>
      
      {success && (
        <div className="success-message">メタデータを保存しました。</div>
      )}
      
      <form onSubmit={handleSubmit} className="metadata-form">
        <div className="form-group">
          <label htmlFor="title">タイトル</label>
          <input
            type="text"
            id="title"
            value={metadata.title}
            onChange={handleTitleChange}
            required
          />
        </div>
        
        <div className="form-group">
          <label>タグ</label>
          <div className="tag-input-container">
            <input
              type="text"
              value={newTag}
              onChange={(e) => setNewTag(e.target.value)}
              placeholder="新しいタグを入力"
            />
            <button
              type="button"
              onClick={handleAddTag}
              className="add-tag-btn"
            >
              追加
            </button>
          </div>
          
          <div className="current-tags">
            <h4>現在のタグ</h4>
            {metadata.tags.length === 0 ? (
              <p>タグが設定されていません。</p>
            ) : (
              <div className="tags">
                {metadata.tags.map((tag) => (
                  <span key={tag} className="tag">
                    {tag}
                    <button
                      type="button"
                      onClick={() => handleRemoveTag(tag)}
                      className="remove-tag-btn"
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            )}
          </div>
          
          <div className="available-tags">
            <h4>利用可能なタグ</h4>
            {availableTags.length === 0 ? (
              <p>利用可能なタグがありません。</p>
            ) : (
              <div className="tags">
                {availableTags
                  .filter(tag => !metadata.tags.includes(tag))
                  .map((tag) => (
                    <span
                      key={tag}
                      className="tag tag-selectable"
                      onClick={() => handleSelectTag(tag)}
                    >
                      {tag}
                    </span>
                  ))}
              </div>
            )}
          </div>
        </div>
        
        <div className="form-actions">
          <button type="submit" className="save-btn">保存</button>
          <button
            type="button"
            onClick={() => navigate(`/view/${filename}`)}
            className="cancel-btn"
          >
            キャンセル
          </button>
        </div>
      </form>
    </div>
  );
}

export default MetadataEditor; 