import React, { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import ReactMarkdown from 'react-markdown';

function DocumentViewer() {
  const { filename } = useParams();
  const navigate = useNavigate();
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    fetchDocument();
  }, [filename]);

  const fetchDocument = async () => {
    try {
      setLoading(true);
      const response = await fetch(`http://localhost:3000/api/wiki/${filename}`);
      
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

  const handleEditDocument = () => {
    navigate(`/edit/${filename}`);
  };

  if (loading) {
    return <div className="loading">Loading document...</div>;
  }

  if (error) {
    return (
      <div className="document-error">
        <h2>Error</h2>
        <p>{error}</p>
        <p>
          <Link to="/">Return to document list</Link>
        </p>
      </div>
    );
  }

  return (
    <div className="document-viewer">
      <div className="viewer-header">
        <h2>{filename}</h2>
        <div className="viewer-actions">
          <Link to={`/edit/${filename}`} className="edit-btn">Edit</Link>
          <Link to={`/history/${filename}`} className="history-btn">History</Link>
          <Link to={`/metadata/${filename}`} className="metadata-btn">Metadata</Link>
        </div>
      </div>
      
      <div className="markdown-content">
        <ReactMarkdown>{content}</ReactMarkdown>
      </div>
    </div>
  );
}

export default DocumentViewer; 