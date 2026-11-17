use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single code fragment within a snippet (supports multi-tab snippets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fragment {
    pub label: String,
    pub language: String,
    pub content: String,
}

/// Represents a tech-stack category for organizing snippets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TechStack {
    React,
    NodeJs,
    Python,
    Rust,
    Go,
    SQL,
    DevOps,
    AiMl,
    Mobile,
    Git,
    DataEngineering,
    Custom(String),
}

impl std::fmt::Display for TechStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TechStack::React => write!(f, "React"),
            TechStack::NodeJs => write!(f, "Node.js"),
            TechStack::Python => write!(f, "Python"),
            TechStack::Rust => write!(f, "Rust"),
            TechStack::Go => write!(f, "Go"),
            TechStack::SQL => write!(f, "SQL"),
            TechStack::DevOps => write!(f, "DevOps"),
            TechStack::AiMl => write!(f, "AI/ML"),
            TechStack::Mobile => write!(f, "Mobile"),
            TechStack::Git => write!(f, "Git"),
            TechStack::DataEngineering => write!(f, "Data Engineering"),
            TechStack::Custom(name) => write!(f, "{name}"),
        }
    }
}

impl TechStack {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "react" => TechStack::React,
            "node" | "nodejs" | "node.js" => TechStack::NodeJs,
            "python" => TechStack::Python,
            "rust" => TechStack::Rust,
            "go" | "golang" => TechStack::Go,
            "sql" => TechStack::SQL,
            "devops" => TechStack::DevOps,
            "ai" | "ml" | "ai/ml" | "aiml" => TechStack::AiMl,
            "mobile" => TechStack::Mobile,
            "git" => TechStack::Git,
            "data" | "data-engineering" | "data_engineering" | "data engineering" => {
                TechStack::DataEngineering
            }
            other => TechStack::Custom(other.to_string()),
        }
    }
}

/// The primary snippet entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub description: String,
    pub fragments: Vec<Fragment>,
    pub tags: Vec<String>,
    pub tech_stack: Option<TechStack>,
    pub folder: Option<String>,
    pub is_favorite: bool,
    pub is_private: bool,
    pub author: Option<String>,
    pub source_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Snippet {
    pub fn new(title: String, language: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description: String::new(),
            fragments: vec![Fragment {
                label: "main".to_string(),
                language,
                content,
            }],
            tags: Vec::new(),
            tech_stack: None,
            folder: None,
            is_favorite: false,
            is_private: false,
            author: None,
            source_url: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_tech_stack(mut self, tech_stack: TechStack) -> Self {
        self.tech_stack = Some(tech_stack);
        self
    }

    pub fn with_folder(mut self, folder: String) -> Self {
        self.folder = Some(folder);
        self
    }

    pub fn add_fragment(&mut self, label: String, language: String, content: String) {
        self.fragments.push(Fragment {
            label,
            language,
            content,
        });
        self.updated_at = Utc::now();
    }

    /// Returns the primary language (from the first fragment).
    pub fn primary_language(&self) -> &str {
        self.fragments
            .first()
            .map(|f| f.language.as_str())
            .unwrap_or("plaintext")
    }

    /// Returns all content concatenated for search indexing.
    pub fn searchable_content(&self) -> String {
        let mut parts = vec![self.title.clone(), self.description.clone()];
        parts.extend(self.tags.iter().cloned());
        parts.extend(self.fragments.iter().map(|f| f.content.clone()));
        if let Some(ref stack) = self.tech_stack {
            parts.push(stack.to_string());
        }
        parts.join(" ")
    }
}

/// Parameters for creating a new snippet.
#[derive(Debug, Clone)]
pub struct CreateSnippetParams {
    pub title: String,
    pub description: String,
    pub language: String,
    pub content: String,
    pub tags: Vec<String>,
    pub tech_stack: Option<TechStack>,
    pub folder: Option<String>,
}

/// Parameters for updating a snippet.
#[derive(Debug, Clone, Default)]
pub struct UpdateSnippetParams {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub tech_stack: Option<TechStack>,
    pub folder: Option<String>,
    pub is_favorite: Option<bool>,
    pub is_private: Option<bool>,
}

/// Filtering options for listing snippets.
#[derive(Debug, Clone, Default)]
pub struct SnippetFilter {
    pub language: Option<String>,
    pub tech_stack: Option<TechStack>,
    pub tags: Option<Vec<String>>,
    pub folder: Option<String>,
    pub favorites_only: bool,
    pub query: Option<String>,
}
