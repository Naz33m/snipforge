use thiserror::Error;

#[derive(Error, Debug)]
pub enum SnipForgeError {
    #[error("Snippet not found: {0}")]
    NotFound(String),

    #[error("Duplicate snippet: {0}")]
    Duplicate(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SnipForgeError>;
