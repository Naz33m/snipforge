import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./styles/App.css";

interface Fragment {
  label: string;
  language: string;
  content: string;
}

interface Snippet {
  id: string;
  title: string;
  description: string;
  fragments: Fragment[];
  tags: string[];
  tech_stack: string | null;
  folder: string | null;
  is_favorite: boolean;
  created_at: string;
  updated_at: string;
}

interface SearchResult {
  snippet: Snippet;
  score: number;
  match_reason: string;
}

interface Stats {
  total: number;
  favorites: number;
  by_language: [string, number][];
  by_tech_stack: [string, number][];
}

type View = "list" | "detail" | "add";

const TECH_STACKS = [
  "React",
  "Node.js",
  "Python",
  "Rust",
  "Go",
  "SQL",
  "DevOps",
  "AI/ML",
  "Mobile",
  "Git",
  "Data Engineering",
];

function App() {
  const [view, setView] = useState<View>("list");
  const [snippets, setSnippets] = useState<Snippet[]>([]);
  const [searchResults, setSearchResults] = useState<SearchResult[] | null>(
    null,
  );
  const [selectedSnippet, setSelectedSnippet] = useState<Snippet | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [filterStack, setFilterStack] = useState<string>("");
  const [stats, setStats] = useState<Stats | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Add form state
  const [addTitle, setAddTitle] = useState("");
  const [addLanguage, setAddLanguage] = useState("");
  const [addContent, setAddContent] = useState("");
  const [addDescription, setAddDescription] = useState("");
  const [addTags, setAddTags] = useState("");
  const [addStack, setAddStack] = useState("");

  const loadSnippets = useCallback(async () => {
    try {
      const result = await invoke<Snippet[]>("list_snippets", {
        techStack: filterStack || null,
        language: null,
        tag: null,
        favoritesOnly: null,
      });
      setSnippets(result);
      setSearchResults(null);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, [filterStack]);

  const loadStats = async () => {
    try {
      const result = await invoke<Stats>("get_stats");
      setStats(result);
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    loadSnippets();
    loadStats();
  }, [loadSnippets]);

  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      setSearchResults(null);
      loadSnippets();
      return;
    }
    try {
      const results = await invoke<SearchResult[]>("search_snippets", {
        query: searchQuery,
        limit: 20,
      });
      setSearchResults(results);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleSearchKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleSearch();
    }
  };

  const handleCreate = async () => {
    if (!addTitle.trim() || !addLanguage.trim() || !addContent.trim()) {
      setError("Title, language, and content are required");
      return;
    }
    try {
      await invoke<Snippet>("create_snippet", {
        args: {
          title: addTitle,
          language: addLanguage,
          content: addContent,
          description: addDescription || null,
          tags: addTags
            ? addTags.split(",").map((t) => t.trim())
            : null,
          tech_stack: addStack || null,
          folder: null,
        },
      });
      setAddTitle("");
      setAddLanguage("");
      setAddContent("");
      setAddDescription("");
      setAddTags("");
      setAddStack("");
      setView("list");
      loadSnippets();
      loadStats();
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await invoke("delete_snippet", { id });
      setSelectedSnippet(null);
      setView("list");
      loadSnippets();
      loadStats();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleToggleFavorite = async (id: string) => {
    try {
      const updated = await invoke<Snippet>("toggle_favorite", { id });
      if (selectedSnippet?.id === id) {
        setSelectedSnippet(updated);
      }
      loadSnippets();
      loadStats();
    } catch (e) {
      setError(String(e));
    }
  };

  const openDetail = (snippet: Snippet) => {
    setSelectedSnippet(snippet);
    setView("detail");
  };

  const displaySnippets = searchResults
    ? searchResults.map((r) => r.snippet)
    : snippets;

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-left">
          <h1 onClick={() => { setView("list"); setSearchResults(null); setSearchQuery(""); }}>
            SnipForge
          </h1>
          <span className="subtitle">AI-Powered Code Snippet Manager</span>
        </div>
        <div className="header-right">
          {stats && (
            <span className="stats-badge">
              {stats.total} snippet{stats.total !== 1 ? "s" : ""} ·{" "}
              {stats.favorites} ★
            </span>
          )}
          <button
            className="btn btn-primary"
            onClick={() => setView("add")}
          >
            + New Snippet
          </button>
        </div>
      </header>

      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button onClick={() => setError(null)}>✕</button>
        </div>
      )}

      <main className="app-main">
        {view === "list" && (
          <>
            <div className="toolbar">
              <div className="search-bar">
                <input
                  type="text"
                  placeholder="Search snippets... (press Enter)"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onKeyDown={handleSearchKeyDown}
                  className="search-input"
                />
                <button className="btn btn-search" onClick={handleSearch}>
                  Search
                </button>
              </div>
              <select
                className="filter-select"
                value={filterStack}
                onChange={(e) => setFilterStack(e.target.value)}
              >
                <option value="">All Stacks</option>
                {TECH_STACKS.map((stack) => (
                  <option key={stack} value={stack.toLowerCase().replace(/[/.]/g, "")}>
                    {stack}
                  </option>
                ))}
              </select>
            </div>

            {searchResults && (
              <div className="search-info">
                Found {searchResults.length} result
                {searchResults.length !== 1 ? "s" : ""} for "{searchQuery}"
                <button
                  className="btn-link"
                  onClick={() => {
                    setSearchResults(null);
                    setSearchQuery("");
                    loadSnippets();
                  }}
                >
                  Clear
                </button>
              </div>
            )}

            <div className="snippet-list">
              {displaySnippets.length === 0 ? (
                <div className="empty-state">
                  <h2>
                    {searchResults ? "No results found" : "No snippets yet"}
                  </h2>
                  <p>
                    {searchResults
                      ? "Try a different search query."
                      : "Click '+ New Snippet' to add your first snippet."}
                  </p>
                </div>
              ) : (
                displaySnippets.map((snippet) => (
                  <div
                    key={snippet.id}
                    className="snippet-card"
                    onClick={() => openDetail(snippet)}
                  >
                    <div className="snippet-header">
                      <div className="snippet-title-row">
                        {snippet.is_favorite && (
                          <span className="favorite-star">★</span>
                        )}
                        <h3>{snippet.title}</h3>
                      </div>
                      <div className="snippet-badges">
                        <span className="language-badge">
                          {snippet.fragments[0]?.language || "unknown"}
                        </span>
                        {snippet.tech_stack && (
                          <span className="stack-badge">
                            {snippet.tech_stack}
                          </span>
                        )}
                      </div>
                    </div>
                    {snippet.description && (
                      <p className="snippet-description">
                        {snippet.description}
                      </p>
                    )}
                    <pre className="snippet-code">
                      <code>
                        {snippet.fragments[0]?.content.slice(0, 200)}
                        {(snippet.fragments[0]?.content.length || 0) > 200
                          ? "..."
                          : ""}
                      </code>
                    </pre>
                    <div className="snippet-footer">
                      <div className="snippet-tags">
                        {snippet.tags.map((tag) => (
                          <span key={tag} className="tag">
                            {tag}
                          </span>
                        ))}
                      </div>
                      {snippet.fragments.length > 1 && (
                        <span className="fragment-count">
                          {snippet.fragments.length} fragments
                        </span>
                      )}
                    </div>
                  </div>
                ))
              )}
            </div>
          </>
        )}

        {view === "detail" && selectedSnippet && (
          <div className="detail-view">
            <div className="detail-header">
              <button
                className="btn btn-back"
                onClick={() => setView("list")}
              >
                ← Back
              </button>
              <div className="detail-actions">
                <button
                  className={`btn btn-fav ${selectedSnippet.is_favorite ? "active" : ""}`}
                  onClick={() => handleToggleFavorite(selectedSnippet.id)}
                >
                  {selectedSnippet.is_favorite ? "★ Favorited" : "☆ Favorite"}
                </button>
                <button
                  className="btn btn-danger"
                  onClick={() => handleDelete(selectedSnippet.id)}
                >
                  Delete
                </button>
              </div>
            </div>

            <div className="detail-content">
              <h2>{selectedSnippet.title}</h2>
              {selectedSnippet.description && (
                <p className="detail-description">
                  {selectedSnippet.description}
                </p>
              )}

              <div className="detail-meta">
                {selectedSnippet.tech_stack && (
                  <span className="stack-badge">
                    {selectedSnippet.tech_stack}
                  </span>
                )}
                <span className="meta-text">
                  Created{" "}
                  {new Date(selectedSnippet.created_at).toLocaleDateString()}
                </span>
              </div>

              {selectedSnippet.tags.length > 0 && (
                <div className="snippet-tags">
                  {selectedSnippet.tags.map((tag) => (
                    <span key={tag} className="tag">
                      {tag}
                    </span>
                  ))}
                </div>
              )}

              <div className="fragments">
                {selectedSnippet.fragments.map((fragment, i) => (
                  <div key={i} className="fragment">
                    <div className="fragment-header">
                      <span className="fragment-label">{fragment.label}</span>
                      <span className="language-badge">
                        {fragment.language}
                      </span>
                    </div>
                    <pre className="snippet-code">
                      <code>{fragment.content}</code>
                    </pre>
                  </div>
                ))}
              </div>
            </div>

            <div className="detail-id">ID: {selectedSnippet.id}</div>
          </div>
        )}

        {view === "add" && (
          <div className="add-view">
            <div className="add-header">
              <button
                className="btn btn-back"
                onClick={() => setView("list")}
              >
                ← Back
              </button>
              <h2>New Snippet</h2>
            </div>

            <div className="add-form">
              <div className="form-group">
                <label>Title *</label>
                <input
                  type="text"
                  value={addTitle}
                  onChange={(e) => setAddTitle(e.target.value)}
                  placeholder="e.g., React useEffect cleanup"
                  className="form-input"
                />
              </div>

              <div className="form-row">
                <div className="form-group">
                  <label>Language *</label>
                  <input
                    type="text"
                    value={addLanguage}
                    onChange={(e) => setAddLanguage(e.target.value)}
                    placeholder="e.g., typescript"
                    className="form-input"
                  />
                </div>
                <div className="form-group">
                  <label>Tech Stack</label>
                  <select
                    value={addStack}
                    onChange={(e) => setAddStack(e.target.value)}
                    className="form-input"
                  >
                    <option value="">None</option>
                    {TECH_STACKS.map((stack) => (
                      <option
                        key={stack}
                        value={stack.toLowerCase().replace(/[/.]/g, "")}
                      >
                        {stack}
                      </option>
                    ))}
                  </select>
                </div>
              </div>

              <div className="form-group">
                <label>Description</label>
                <input
                  type="text"
                  value={addDescription}
                  onChange={(e) => setAddDescription(e.target.value)}
                  placeholder="Brief description of what this snippet does"
                  className="form-input"
                />
              </div>

              <div className="form-group">
                <label>Code *</label>
                <textarea
                  value={addContent}
                  onChange={(e) => setAddContent(e.target.value)}
                  placeholder="Paste your code here..."
                  className="form-textarea"
                  rows={12}
                />
              </div>

              <div className="form-group">
                <label>Tags (comma-separated)</label>
                <input
                  type="text"
                  value={addTags}
                  onChange={(e) => setAddTags(e.target.value)}
                  placeholder="e.g., hooks, cleanup, lifecycle"
                  className="form-input"
                />
              </div>

              <div className="form-actions">
                <button className="btn btn-primary" onClick={handleCreate}>
                  Save Snippet
                </button>
                <button
                  className="btn btn-secondary"
                  onClick={() => setView("list")}
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;
