mod sqlite;

pub use sqlite::SqliteStorage;

use crate::error::Result;
use crate::models::{CreateSnippetParams, Snippet, SnippetFilter, UpdateSnippetParams};

/// Trait defining the storage backend interface.
pub trait SnippetStore: Send + Sync {
    fn init(&self) -> Result<()>;
    fn create(&self, params: CreateSnippetParams) -> Result<Snippet>;
    fn get(&self, id: &str) -> Result<Snippet>;
    fn update(&self, id: &str, params: UpdateSnippetParams) -> Result<Snippet>;
    fn delete(&self, id: &str) -> Result<()>;
    fn list(&self, filter: SnippetFilter) -> Result<Vec<Snippet>>;
    fn count(&self) -> Result<usize>;
}
