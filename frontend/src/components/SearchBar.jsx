import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';

function SearchBar() {
  const [query, setQuery] = useState('');
  const [isSearching, setIsSearching] = useState(false);
  const navigate = useNavigate();

  const handleSubmit = (e) => {
    e.preventDefault();
    if (query.trim()) {
      setIsSearching(true);
      navigate(`/search?q=${encodeURIComponent(query)}`);
      setIsSearching(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="search-bar">
      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="ドキュメントを検索..."
        disabled={isSearching}
      />
      <button type="submit" disabled={isSearching || !query.trim()}>
        {isSearching ? '検索中...' : '検索'}
      </button>
    </form>
  );
}

export default SearchBar; 