import React from 'react';
import { Routes, Route, Link } from 'react-router-dom';
import DocumentList from './components/DocumentList';
import DocumentEditor from './components/DocumentEditor';
import DocumentViewer from './components/DocumentViewer';

function App() {
  return (
    <div className="app-container">
      <header className="app-header">
        <h1>MD Wiki Rust</h1>
        <nav>
          <Link to="/">Home</Link>
        </nav>
      </header>
      
      <main className="app-content">
        <Routes>
          <Route path="/" element={<DocumentList />} />
          <Route path="/edit/:filename" element={<DocumentEditor />} />
          <Route path="/view/:filename" element={<DocumentViewer />} />
        </Routes>
      </main>
      
      <footer className="app-footer">
        <p>MD Wiki Rust - A Markdown Wiki built with Rust and Tauri</p>
      </footer>
    </div>
  );
}

export default App; 