use crate::error::Result;
use crate::models::Snippet;

/// Search result with relevance score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub snippet: Snippet,
    pub score: f64,
    pub match_reason: MatchReason,
}

#[derive(Debug, Clone)]
pub enum MatchReason {
    TitleMatch,
    DescriptionMatch,
    ContentMatch,
    TagMatch,
    SemanticMatch,
    Combined,
}

impl std::fmt::Display for MatchReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchReason::TitleMatch => write!(f, "title"),
            MatchReason::DescriptionMatch => write!(f, "description"),
            MatchReason::ContentMatch => write!(f, "content"),
            MatchReason::TagMatch => write!(f, "tag"),
            MatchReason::SemanticMatch => write!(f, "semantic"),
            MatchReason::Combined => write!(f, "combined"),
        }
    }
}

/// Trait for search backends (keyword, semantic, hybrid).
pub trait SearchEngine: Send + Sync {
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    fn index_snippet(&self, snippet: &Snippet) -> Result<()>;
    fn remove_snippet(&self, id: &str) -> Result<()>;
    fn reindex_all(&self, snippets: &[Snippet]) -> Result<()>;
}

/// Simple in-memory keyword search (used as fallback and for basic matching).
pub struct KeywordSearch {
    snippets: std::sync::RwLock<Vec<Snippet>>,
}

impl KeywordSearch {
    pub fn new() -> Self {
        Self {
            snippets: std::sync::RwLock::new(Vec::new()),
        }
    }

    fn score_snippet(snippet: &Snippet, query: &str) -> Option<(f64, MatchReason)> {
        let query_lower = query.to_lowercase();
        let terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut score = 0.0;
        let mut reason = MatchReason::Combined;

        // Title match (highest weight)
        let title_lower = snippet.title.to_lowercase();
        if title_lower.contains(&query_lower) {
            score += 10.0;
            reason = MatchReason::TitleMatch;
        } else {
            let title_term_hits = terms.iter().filter(|t| title_lower.contains(*t)).count();
            score += (title_term_hits as f64 / terms.len() as f64) * 8.0;
        }

        // Description match
        let desc_lower = snippet.description.to_lowercase();
        if desc_lower.contains(&query_lower) {
            score += 5.0;
            if score < 6.0 {
                reason = MatchReason::DescriptionMatch;
            }
        }

        // Tag match
        let tag_hits = snippet
            .tags
            .iter()
            .filter(|tag| terms.iter().any(|t| tag.to_lowercase().contains(t)))
            .count();
        if tag_hits > 0 {
            score += tag_hits as f64 * 3.0;
            if score < 4.0 {
                reason = MatchReason::TagMatch;
            }
        }

        // Content match
        for fragment in &snippet.fragments {
            let content_lower = fragment.content.to_lowercase();
            let content_hits = terms.iter().filter(|t| content_lower.contains(*t)).count();
            score += (content_hits as f64 / terms.len() as f64) * 2.0;
            if content_hits > 0 && score < 3.0 {
                reason = MatchReason::ContentMatch;
            }
        }

        if score > 0.0 {
            Some((score, reason))
        } else {
            None
        }
    }
}

impl Default for KeywordSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine for KeywordSearch {
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let snippets = self
            .snippets
            .read()
            .map_err(|e| crate::error::SnipForgeError::Search(format!("Lock error: {e}")))?;

        let mut results: Vec<SearchResult> = snippets
            .iter()
            .filter_map(|snippet| {
                Self::score_snippet(snippet, query).map(|(score, reason)| SearchResult {
                    snippet: snippet.clone(),
                    score,
                    match_reason: reason,
                })
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    fn index_snippet(&self, snippet: &Snippet) -> Result<()> {
        let mut snippets = self
            .snippets
            .write()
            .map_err(|e| crate::error::SnipForgeError::Search(format!("Lock error: {e}")))?;
        snippets.retain(|s| s.id != snippet.id);
        snippets.push(snippet.clone());
        Ok(())
    }

    fn remove_snippet(&self, id: &str) -> Result<()> {
        let mut snippets = self
            .snippets
            .write()
            .map_err(|e| crate::error::SnipForgeError::Search(format!("Lock error: {e}")))?;
        snippets.retain(|s| s.id != id);
        Ok(())
    }

    fn reindex_all(&self, new_snippets: &[Snippet]) -> Result<()> {
        let mut snippets = self
            .snippets
            .write()
            .map_err(|e| crate::error::SnipForgeError::Search(format!("Lock error: {e}")))?;
        *snippets = new_snippets.to_vec();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TechStack;

    fn sample_snippets() -> Vec<Snippet> {
        vec![
            Snippet::new(
                "Validate Email".to_string(),
                "python".to_string(),
                "import re\ndef validate_email(email): pass".to_string(),
            )
            .with_description("Email validation using regex".to_string())
            .with_tags(vec!["validation".to_string(), "email".to_string()])
            .with_tech_stack(TechStack::Python),
            Snippet::new(
                "React useEffect Hook".to_string(),
                "typescript".to_string(),
                "useEffect(() => { fetchData(); }, [])".to_string(),
            )
            .with_description("Side effect hook pattern".to_string())
            .with_tags(vec!["hooks".to_string(), "react".to_string()])
            .with_tech_stack(TechStack::React),
            Snippet::new(
                "Docker Compose PostgreSQL".to_string(),
                "yaml".to_string(),
                "services:\n  db:\n    image: postgres:15".to_string(),
            )
            .with_description("PostgreSQL Docker setup".to_string())
            .with_tags(vec!["docker".to_string(), "postgres".to_string()])
            .with_tech_stack(TechStack::DevOps),
        ]
    }

    #[test]
    fn test_keyword_search_title_match() {
        let engine = KeywordSearch::new();
        engine.reindex_all(&sample_snippets()).unwrap();

        let results = engine.search("validate email", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].snippet.title, "Validate Email");
    }

    #[test]
    fn test_keyword_search_tag_match() {
        let engine = KeywordSearch::new();
        engine.reindex_all(&sample_snippets()).unwrap();

        let results = engine.search("hooks", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].snippet.title, "React useEffect Hook");
    }

    #[test]
    fn test_keyword_search_no_results() {
        let engine = KeywordSearch::new();
        engine.reindex_all(&sample_snippets()).unwrap();

        let results = engine.search("zzzznonexistent", 10).unwrap();
        assert!(results.is_empty());
    }
}
