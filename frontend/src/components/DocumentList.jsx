import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

function DocumentList() {
  const [documents, setDocuments] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [newDocName, setNewDocName] = useState('');

  useEffect(() => {
    fetchDocuments();
  }, []);

  const fetchDocuments = async () => {
    try {
      setLoading(true);
      const response = await fetch(`${API_BASE_URL}/api/documents`);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      setDocuments(data.documents);
      setError(null);
    } catch (err) {
      setError(`Failed to fetch documents: ${err.message}`);
      console.error('Error fetching documents:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateDocument = async (e) => {
    e.preventDefault();
    if (!newDocName.trim()) return;
    
    try {
      const response = await fetch(`${API_BASE_URL}/api/documents/${newDocName}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          filename: newDocName,
          content: `# ${newDocName}\n\nNew document created.`,
        }),
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      setNewDocName('');
      fetchDocuments();
    } catch (err) {
      setError(`Failed to create document: ${err.message}`);
      console.error('Error creating document:', err);
    }
  };


  return (
    <div className="document-list">
      <h2>Wiki Documents</h2>
      
      {error && <div className="error-message">{error}</div>}
      
      <form onSubmit={handleCreateDocument} className="new-document-form">
        <input
          type="text"
          value={newDocName}
          onChange={(e) => setNewDocName(e.target.value)}
          placeholder="New document name"
          required
        />
        <button type="submit">Create</button>
      </form>
      
      {loading ? (
        <p>Loading documents...</p>
      ) : documents.length === 0 ? (
        <p>No documents found. Create your first document above.</p>
      ) : (
        <ul className="documents">
          {documents.map((doc) => (
            <li key={doc} className="document-item">
              <span className="document-name">{doc}</span>
              <div className="document-actions">
                <Link to={`/view/${doc}`} className="view-btn">View</Link>
                <Link to={`/edit/${doc}`} className="edit-btn">Edit</Link>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default DocumentList; 