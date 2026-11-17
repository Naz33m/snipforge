import { useState } from "react";
import "./styles/App.css";

interface Snippet {
  id: string;
  title: string;
  description: string;
  language: string;
  content: string;
  tags: string[];
  tech_stack?: string;
}

function App() {
  const [searchQuery, setSearchQuery] = useState("");
  const [snippets] = useState<Snippet[]>([]);

  return (
    <div className="app">
      <header className="app-header">
        <h1>SnipForge</h1>
        <p className="subtitle">AI-Powered Code Snippet Manager</p>
      </header>

      <main className="app-main">
        <div className="search-bar">
          <input
            type="text"
            placeholder="Search snippets... (e.g., 'validate email python')"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>

        <div className="snippet-list">
          {snippets.length === 0 ? (
            <div className="empty-state">
              <h2>No snippets yet</h2>
              <p>
                Add your first snippet using the CLI or click the + button.
              </p>
              <code>
                snipforge add --title "My Snippet" --language python --content
                "print('hello')"
              </code>
            </div>
          ) : (
            snippets.map((snippet) => (
              <div key={snippet.id} className="snippet-card">
                <div className="snippet-header">
                  <h3>{snippet.title}</h3>
                  <span className="language-badge">{snippet.language}</span>
                </div>
                {snippet.description && (
                  <p className="snippet-description">{snippet.description}</p>
                )}
                <pre className="snippet-code">
                  <code>{snippet.content}</code>
                </pre>
                <div className="snippet-tags">
                  {snippet.tags.map((tag) => (
                    <span key={tag} className="tag">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            ))
          )}
        </div>
      </main>
    </div>
  );
}

export default App;
