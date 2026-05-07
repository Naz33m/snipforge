pub mod ai;
pub mod error;
pub mod models;
pub mod search;
pub mod storage;

use error::Result;
use models::*;
use search::{KeywordSearch, SearchEngine, SearchResult};
use storage::{SnippetStore, SqliteStorage};

use std::path::PathBuf;
use std::sync::Arc;

/// The main SnipForge engine that ties together storage, search, and AI.
pub struct SnipForge {
    store: Arc<dyn SnippetStore>,
    search: Arc<dyn SearchEngine>,
    data_dir: PathBuf,
}

impl SnipForge {
    /// Create a new SnipForge instance with the given data directory.
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("snippets.db");
        let store = Arc::new(SqliteStorage::new(&db_path)?);

        let search = Arc::new(KeywordSearch::new());

        // Index existing snippets into the search engine.
        let existing = store.list(SnippetFilter::default())?;
        search.reindex_all(&existing)?;

        Ok(Self {
            store,
            search,
            data_dir,
        })
    }

    /// Create an in-memory instance for testing.
    pub fn in_memory() -> Result<Self> {
        let store = Arc::new(SqliteStorage::in_memory()?);
        let search = Arc::new(KeywordSearch::new());

        Ok(Self {
            store,
            search,
            data_dir: PathBuf::from("/tmp/snipforge-test"),
        })
    }

    /// Returns the data directory path.
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Create a new snippet.
    pub fn create_snippet(&self, params: CreateSnippetParams) -> Result<Snippet> {
        let snippet = self.store.create(params)?;
        self.search.index_snippet(&snippet)?;
        Ok(snippet)
    }

    /// Get a snippet by ID.
    pub fn get_snippet(&self, id: &str) -> Result<Snippet> {
        self.store.get(id)
    }

    /// Update a snippet.
    pub fn update_snippet(&self, id: &str, params: UpdateSnippetParams) -> Result<Snippet> {
        let snippet = self.store.update(id, params)?;
        self.search.index_snippet(&snippet)?;
        Ok(snippet)
    }

    /// Delete a snippet.
    pub fn delete_snippet(&self, id: &str) -> Result<()> {
        self.store.delete(id)?;
        self.search.remove_snippet(id)?;
        Ok(())
    }

    /// List snippets with optional filters.
    pub fn list_snippets(&self, filter: SnippetFilter) -> Result<Vec<Snippet>> {
        self.store.list(filter)
    }

    /// Search snippets using the search engine (keyword + future semantic).
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.search.search(query, limit)
    }

    /// Get total snippet count.
    pub fn count(&self) -> Result<usize> {
        self.store.count()
    }

    /// Export all snippets as JSON.
    pub fn export_json(&self) -> Result<String> {
        let snippets = self.store.list(SnippetFilter::default())?;
        Ok(serde_json::to_string_pretty(&snippets)?)
    }

    /// Import snippets from JSON.
    pub fn import_json(&self, json: &str) -> Result<usize> {
        let snippets: Vec<Snippet> = serde_json::from_str(json)?;
        let mut count = 0;

        for snippet in snippets {
            let primary_lang = snippet.primary_language().to_string();
            let primary_content = snippet
                .fragments
                .first()
                .map(|f| f.content.clone())
                .unwrap_or_default();

            let params = CreateSnippetParams {
                title: snippet.title,
                description: snippet.description,
                language: primary_lang,
                content: primary_content,
                tags: snippet.tags,
                tech_stack: snippet.tech_stack,
                folder: snippet.folder,
            };

            self.create_snippet(params)?;
            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_lifecycle() {
        let engine = SnipForge::in_memory().unwrap();

        let snippet = engine
            .create_snippet(CreateSnippetParams {
                title: "Test Snippet".to_string(),
                description: "A test".to_string(),
                language: "rust".to_string(),
                content: "fn test() { assert!(true); }".to_string(),
                tags: vec!["test".to_string()],
                tech_stack: Some(TechStack::Rust),
                folder: None,
            })
            .unwrap();

        assert_eq!(engine.count().unwrap(), 1);

        let fetched = engine.get_snippet(&snippet.id).unwrap();
        assert_eq!(fetched.title, "Test Snippet");

        let results = engine.search("test", 10).unwrap();
        assert!(!results.is_empty());

        engine.delete_snippet(&snippet.id).unwrap();
        assert_eq!(engine.count().unwrap(), 0);
    }

    #[test]
    fn test_export_import() {
        let engine = SnipForge::in_memory().unwrap();

        engine
            .create_snippet(CreateSnippetParams {
                title: "Export Me".to_string(),
                description: "For export".to_string(),
                language: "python".to_string(),
                content: "print('exported')".to_string(),
                tags: vec!["export".to_string()],
                tech_stack: Some(TechStack::Python),
                folder: None,
            })
            .unwrap();

        let json = engine.export_json().unwrap();

        let engine2 = SnipForge::in_memory().unwrap();
        let imported = engine2.import_json(&json).unwrap();
        assert_eq!(imported, 1);
        assert_eq!(engine2.count().unwrap(), 1);
    }
}
