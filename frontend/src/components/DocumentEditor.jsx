import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

function DocumentEditor() {
  const { filename } = useParams();
  const navigate = useNavigate();
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    fetchDocument();
  }, [filename]);

  const fetchDocument = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${API_BASE_URL}/api/documents/${filename}`);
      
      if (response.status === 404) {
        // Create a new document if it doesn't exist
        setContent(`# ${filename}\n\nStart writing your document here...`);
        setLoading(false);
        return;
      }
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      setContent(data.content);
      setError(null);
    } catch (err) {
      setError(`Failed to fetch document: ${err.message}`);
      console.error('Error fetching document:', err);
    } finally {
      setLoading(false);
    }
  };

  const saveDocument = async () => {
    try {
      setSaved(false);
      const response = await fetch(`${API_BASE_URL}/api/documents/${filename}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          filename,
          content,
        }),
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      setError(`Failed to save document: ${err.message}`);
      console.error('Error saving document:', err);
    }
  };


  const handleViewDocument = () => {
    navigate(`/view/${filename}`);
  };

  if (loading) {
    return <div className="loading">Loading document...</div>;
  }

  return (
    <div className="document-editor">
      <div className="editor-header">
        <h2>Editing: {filename}</h2>
        <div className="editor-actions">
          <button onClick={saveDocument} className="save-btn">Save</button>
          <button onClick={handleViewDocument} className="view-btn">View</button>
        </div>
      </div>
      
      {error && <div className="error-message">{error}</div>}
      {saved && <div className="success-message">Document saved successfully!</div>}
      
      <div className="editor-container">
        <textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          className="markdown-editor"
          placeholder="Start writing your markdown here..."
        />
      </div>
    </div>
  );
}

export default DocumentEditor; 