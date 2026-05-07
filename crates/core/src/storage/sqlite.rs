use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

use crate::error::{Result, SnipForgeError};
use crate::models::*;
use crate::storage::SnippetStore;

pub struct SqliteStorage {
    conn: Mutex<Connection>,
}

impl SqliteStorage {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init()?;
        Ok(store)
    }

    fn snippet_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Snippet> {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let description: String = row.get(2)?;
        let fragments_json: String = row.get(3)?;
        let tags_json: String = row.get(4)?;
        let tech_stack_str: Option<String> = row.get(5)?;
        let folder: Option<String> = row.get(6)?;
        let is_favorite: bool = row.get(7)?;
        let is_private: bool = row.get(8)?;
        let author: Option<String> = row.get(9)?;
        let source_url: Option<String> = row.get(10)?;
        let created_at_str: String = row.get(11)?;
        let updated_at_str: String = row.get(12)?;

        let fragments: Vec<Fragment> = serde_json::from_str(&fragments_json).unwrap_or_default();
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let tech_stack = tech_stack_str.map(|s| TechStack::parse(&s));
        let created_at = created_at_str
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now());
        let updated_at = updated_at_str
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now());

        Ok(Snippet {
            id,
            title,
            description,
            fragments,
            tags,
            tech_stack,
            folder,
            is_favorite,
            is_private,
            author,
            source_url,
            created_at,
            updated_at,
        })
    }
}

impl SnippetStore for SqliteStorage {
    fn init(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| SnipForgeError::Storage(format!("Failed to acquire lock: {e}")))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snippets (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                fragments TEXT NOT NULL DEFAULT '[]',
                tags TEXT NOT NULL DEFAULT '[]',
                tech_stack TEXT,
                folder TEXT,
                is_favorite INTEGER NOT NULL DEFAULT 0,
                is_private INTEGER NOT NULL DEFAULT 0,
                author TEXT,
                source_url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_snippets_tech_stack ON snippets(tech_stack);
            CREATE INDEX IF NOT EXISTS idx_snippets_folder ON snippets(folder);
            CREATE INDEX IF NOT EXISTS idx_snippets_created_at ON snippets(created_at);

            CREATE VIRTUAL TABLE IF NOT EXISTS snippets_fts USING fts5(
                id UNINDEXED,
                title,
                description,
                content,
                tags
            );",
        )?;
        Ok(())
    }

    fn create(&self, params: CreateSnippetParams) -> Result<Snippet> {
        let snippet = Snippet::new(params.title, params.language, params.content)
            .with_description(params.description)
            .with_tags(params.tags);

        let snippet = if let Some(ts) = params.tech_stack {
            snippet.with_tech_stack(ts)
        } else {
            snippet
        };

        let snippet = if let Some(folder) = params.folder {
            snippet.with_folder(folder)
        } else {
            snippet
        };

        let conn = self
            .conn
            .lock()
            .map_err(|e| SnipForgeError::Storage(format!("Failed to acquire lock: {e}")))?;

        let fragments_json = serde_json::to_string(&snippet.fragments)?;
        let tags_json = serde_json::to_string(&snippet.tags)?;
        let tech_stack_str = snippet.tech_stack.as_ref().map(|ts| ts.to_string());

        conn.execute(
            "INSERT INTO snippets (id, title, description, fragments, tags, tech_stack, folder, is_favorite, is_private, author, source_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                snippet.id,
                snippet.title,
                snippet.description,
                fragments_json,
                tags_json,
                tech_stack_str,
                snippet.folder,
                snippet.is_favorite,
                snippet.is_private,
                snippet.author,
                snippet.source_url,
                snippet.created_at.to_rfc3339(),
                snippet.updated_at.to_rfc3339(),
            ],
        )?;

        let content = snippet
            .fragments
            .iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        conn.execute(
            "INSERT INTO snippets_fts (id, title, description, content, tags)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                snippet.id,
                snippet.title,
                snippet.description,
                content,
                snippet.tags.join(" "),
            ],
        )?;

        Ok(snippet)
    }

    fn get(&self, id: &str) -> Result<Snippet> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| SnipForgeError::Storage(format!("Failed to acquire lock: {e}")))?;

        let mut stmt = conn.prepare(
            "SELECT id, title, description, fragments, tags, tech_stack, folder, is_favorite, is_private, author, source_url, created_at, updated_at
             FROM snippets WHERE id = ?1",
        )?;

        let snippet = stmt
            .query_row(params![id], Self::snippet_from_row)
            .map_err(|_| SnipForgeError::NotFound(id.to_string()))?;

        Ok(snippet)
    }

    fn update(&self, id: &str, params: UpdateSnippetParams) -> Result<Snippet> {
        let mut snippet = self.get(id)?;

        if let Some(title) = params.title {
            snippet.title = title;
        }
        if let Some(description) = params.description {
            snippet.description = description;
        }
        if let Some(tags) = params.tags {
            snippet.tags = tags;
        }
        if let Some(tech_stack) = params.tech_stack {
            snippet.tech_stack = Some(tech_stack);
        }
        if let Some(folder) = params.folder {
            snippet.folder = Some(folder);
        }
        if let Some(is_favorite) = params.is_favorite {
            snippet.is_favorite = is_favorite;
        }
        if let Some(is_private) = params.is_private {
            snippet.is_private = is_private;
        }

        snippet.updated_at = chrono::Utc::now();

        let conn = self
            .conn
            .lock()
            .map_err(|e| SnipForgeError::Storage(format!("Failed to acquire lock: {e}")))?;

        let fragments_json = serde_json::to_string(&snippet.fragments)?;
        let tags_json = serde_json::to_string(&snippet.tags)?;
        let tech_stack_str = snippet.tech_stack.as_ref().map(|ts| ts.to_string());

        conn.execute(
            "UPDATE snippets SET title=?2, description=?3, fragments=?4, tags=?5, tech_stack=?6, folder=?7, is_favorite=?8, is_private=?9, updated_at=?10 WHERE id=?1",
            rusqlite::params![
                snippet.id,
                snippet.title,
                snippet.description,
                fragments_json,
                tags_json,
                tech_stack_str,
                snippet.folder,
                snippet.is_favorite,
                snippet.is_private,
                snippet.updated_at.to_rfc3339(),
            ],
        )?;

        let content = snippet
            .fragments
            .iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        conn.execute(
            "DELETE FROM snippets_fts WHERE id = ?1",
            rusqlite::params![snippet.id],
        )?;
        conn.execute(
            "INSERT INTO snippets_fts (id, title, description, content, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                snippet.id,
                snippet.title,
                snippet.description,
                content,
                snippet.tags.join(" "),
            ],
        )?;

        Ok(snippet)
    }

    fn delete(&self, id: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| SnipForgeError::Storage(format!("Failed to acquire lock: {e}")))?;

        let rows = conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(SnipForgeError::NotFound(id.to_string()));
        }

        conn.execute("DELETE FROM snippets_fts WHERE id = ?1", params![id])?;

        Ok(())
    }

    fn list(&self, filter: SnippetFilter) -> Result<Vec<Snippet>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| SnipForgeError::Storage(format!("Failed to acquire lock: {e}")))?;

        // If there's a text query, use FTS.
        if let Some(ref query) = filter.query {
            let mut stmt = conn.prepare(
                "SELECT s.id, s.title, s.description, s.fragments, s.tags, s.tech_stack, s.folder, s.is_favorite, s.is_private, s.author, s.source_url, s.created_at, s.updated_at
                 FROM snippets s
                 INNER JOIN snippets_fts fts ON s.id = fts.id
                 WHERE snippets_fts MATCH ?1
                 ORDER BY rank",
            )?;

            let snippets = stmt
                .query_map(params![query], Self::snippet_from_row)?
                .filter_map(|r| r.ok())
                .collect();

            return Ok(snippets);
        }

        // Otherwise, build a filtered query.
        let mut sql = String::from(
            "SELECT id, title, description, fragments, tags, tech_stack, folder, is_favorite, is_private, author, source_url, created_at, updated_at FROM snippets WHERE 1=1",
        );
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(ref tech_stack) = filter.tech_stack {
            bind_values.push(tech_stack.to_string());
            sql.push_str(&format!(" AND tech_stack = ?{}", bind_values.len()));
        }
        if let Some(ref folder) = filter.folder {
            bind_values.push(folder.clone());
            sql.push_str(&format!(" AND folder = ?{}", bind_values.len()));
        }
        if let Some(ref language) = filter.language {
            bind_values.push(language.clone());
            sql.push_str(&format!(
                " AND json_extract(fragments, '$[0].language') = ?{}",
                bind_values.len()
            ));
        }
        if filter.favorites_only {
            sql.push_str(" AND is_favorite = 1");
        }

        sql.push_str(" ORDER BY updated_at DESC");

        let mut stmt = conn.prepare(&sql)?;

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = bind_values
            .iter()
            .map(|v| v as &dyn rusqlite::types::ToSql)
            .collect();

        let snippets = stmt
            .query_map(param_refs.as_slice(), Self::snippet_from_row)?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();

        // Post-filter by tags if needed (JSON array matching in SQLite is limited).
        if let Some(ref filter_tags) = filter.tags {
            let filtered = snippets
                .into_iter()
                .filter(|s| filter_tags.iter().all(|t| s.tags.contains(t)))
                .collect();
            return Ok(filtered);
        }

        Ok(snippets)
    }

    fn count(&self) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| SnipForgeError::Storage(format!("Failed to acquire lock: {e}")))?;
        let count: usize = conn.query_row("SELECT COUNT(*) FROM snippets", [], |row| row.get(0))?;
        Ok(count)
    }

    fn import_snippet(&self, snippet: &Snippet) -> Result<Snippet> {
        let new_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let imported = Snippet {
            id: new_id,
            title: snippet.title.clone(),
            description: snippet.description.clone(),
            fragments: snippet.fragments.clone(),
            tags: snippet.tags.clone(),
            tech_stack: snippet.tech_stack.clone(),
            folder: snippet.folder.clone(),
            is_favorite: snippet.is_favorite,
            is_private: snippet.is_private,
            author: snippet.author.clone(),
            source_url: snippet.source_url.clone(),
            created_at: now,
            updated_at: now,
        };

        let conn = self
            .conn
            .lock()
            .map_err(|e| SnipForgeError::Storage(format!("Failed to acquire lock: {e}")))?;

        let fragments_json = serde_json::to_string(&imported.fragments)?;
        let tags_json = serde_json::to_string(&imported.tags)?;
        let tech_stack_str = imported.tech_stack.as_ref().map(|ts| ts.to_string());

        conn.execute(
            "INSERT INTO snippets (id, title, description, fragments, tags, tech_stack, folder, is_favorite, is_private, author, source_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                imported.id,
                imported.title,
                imported.description,
                fragments_json,
                tags_json,
                tech_stack_str,
                imported.folder,
                imported.is_favorite,
                imported.is_private,
                imported.author,
                imported.source_url,
                imported.created_at.to_rfc3339(),
                imported.updated_at.to_rfc3339(),
            ],
        )?;

        let content = imported
            .fragments
            .iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        conn.execute(
            "INSERT INTO snippets_fts (id, title, description, content, tags)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                imported.id,
                imported.title,
                imported.description,
                content,
                imported.tags.join(" "),
            ],
        )?;

        Ok(imported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> SqliteStorage {
        SqliteStorage::in_memory().expect("Failed to create in-memory store")
    }

    #[test]
    fn test_create_and_get_snippet() {
        let store = create_test_store();
        let params = CreateSnippetParams {
            title: "Hello World".to_string(),
            description: "A simple hello world".to_string(),
            language: "python".to_string(),
            content: "print('Hello, World!')".to_string(),
            tags: vec!["beginner".to_string(), "python".to_string()],
            tech_stack: Some(TechStack::Python),
            folder: None,
        };

        let snippet = store.create(params).unwrap();
        assert_eq!(snippet.title, "Hello World");
        assert_eq!(snippet.fragments.len(), 1);
        assert_eq!(snippet.primary_language(), "python");

        let fetched = store.get(&snippet.id).unwrap();
        assert_eq!(fetched.title, snippet.title);
    }

    #[test]
    fn test_update_snippet() {
        let store = create_test_store();
        let params = CreateSnippetParams {
            title: "Original Title".to_string(),
            description: String::new(),
            language: "rust".to_string(),
            content: "fn main() {}".to_string(),
            tags: vec![],
            tech_stack: None,
            folder: None,
        };

        let snippet = store.create(params).unwrap();
        let updated = store
            .update(
                &snippet.id,
                UpdateSnippetParams {
                    title: Some("Updated Title".to_string()),
                    is_favorite: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert!(updated.is_favorite);
    }

    #[test]
    fn test_delete_snippet() {
        let store = create_test_store();
        let params = CreateSnippetParams {
            title: "To Delete".to_string(),
            description: String::new(),
            language: "javascript".to_string(),
            content: "console.log('bye')".to_string(),
            tags: vec![],
            tech_stack: None,
            folder: None,
        };

        let snippet = store.create(params).unwrap();
        store.delete(&snippet.id).unwrap();

        assert!(store.get(&snippet.id).is_err());
    }

    #[test]
    fn test_list_with_filters() {
        let store = create_test_store();

        store
            .create(CreateSnippetParams {
                title: "React Hook".to_string(),
                description: "Custom hook".to_string(),
                language: "typescript".to_string(),
                content: "export function useCounter() {}".to_string(),
                tags: vec!["hooks".to_string(), "react".to_string()],
                tech_stack: Some(TechStack::React),
                folder: Some("frontend".to_string()),
            })
            .unwrap();

        store
            .create(CreateSnippetParams {
                title: "API Route".to_string(),
                description: "Express route".to_string(),
                language: "typescript".to_string(),
                content: "app.get('/api', handler)".to_string(),
                tags: vec!["api".to_string(), "express".to_string()],
                tech_stack: Some(TechStack::NodeJs),
                folder: Some("backend".to_string()),
            })
            .unwrap();

        let all = store.list(SnippetFilter::default()).unwrap();
        assert_eq!(all.len(), 2);

        let react_only = store
            .list(SnippetFilter {
                tech_stack: Some(TechStack::React),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(react_only.len(), 1);
        assert_eq!(react_only[0].title, "React Hook");
    }

    #[test]
    fn test_fulltext_search() {
        let store = create_test_store();

        store
            .create(CreateSnippetParams {
                title: "Validate Email".to_string(),
                description: "Email validation regex".to_string(),
                language: "python".to_string(),
                content: "import re\ndef validate_email(email): ...".to_string(),
                tags: vec!["validation".to_string()],
                tech_stack: Some(TechStack::Python),
                folder: None,
            })
            .unwrap();

        store
            .create(CreateSnippetParams {
                title: "Sort Array".to_string(),
                description: "Sort an array".to_string(),
                language: "javascript".to_string(),
                content: "arr.sort((a, b) => a - b)".to_string(),
                tags: vec!["sorting".to_string()],
                tech_stack: None,
                folder: None,
            })
            .unwrap();

        let results = store
            .list(SnippetFilter {
                query: Some("email".to_string()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Validate Email");
    }

    #[test]
    fn test_count() {
        let store = create_test_store();
        assert_eq!(store.count().unwrap(), 0);

        store
            .create(CreateSnippetParams {
                title: "Test".to_string(),
                description: String::new(),
                language: "rust".to_string(),
                content: "fn test() {}".to_string(),
                tags: vec![],
                tech_stack: None,
                folder: None,
            })
            .unwrap();

        assert_eq!(store.count().unwrap(), 1);
    }
}
