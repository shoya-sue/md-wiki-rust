import React from 'react';
import { Routes, Route, Link } from 'react-router-dom';
import DocumentList from './components/DocumentList';
import DocumentEditor from './components/DocumentEditor';
import DocumentViewer from './components/DocumentViewer';
import DocumentHistory from './components/DocumentHistory';
import DocumentVersionViewer from './components/DocumentVersionViewer';
import SearchBar from './components/SearchBar';
import SearchResults from './components/SearchResults';

function App() {
  return (
    <div className="app-container">
      <header className="app-header">
        <h1>MD Wiki Rust</h1>
        <div className="header-right">
          <SearchBar />
          <nav>
            <Link to="/">Home</Link>
          </nav>
        </div>
      </header>
      
      <main className="app-content">
        <Routes>
          <Route path="/" element={<DocumentList />} />
          <Route path="/edit/:filename" element={<DocumentEditor />} />
          <Route path="/view/:filename" element={<DocumentViewer />} />
          <Route path="/history/:filename" element={<DocumentHistory />} />
          <Route path="/view/:filename/version/:commitId" element={<DocumentVersionViewer />} />
          <Route path="/search" element={<SearchResults />} />
        </Routes>
      </main>
      
      <footer className="app-footer">
        <p>MD Wiki Rust - A Markdown Wiki built with Rust and Tauri</p>
      </footer>
    </div>
  );
}

export default App; 