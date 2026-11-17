use crate::error::Result;
use crate::models::{Snippet, TechStack};

/// AI-generated enrichments for a snippet.
#[derive(Debug, Clone)]
pub struct SnippetEnrichment {
    pub suggested_title: Option<String>,
    pub suggested_description: Option<String>,
    pub suggested_tags: Vec<String>,
    pub detected_language: Option<String>,
    pub detected_tech_stack: Option<TechStack>,
    pub complexity: Complexity,
    pub has_sensitive_data: bool,
    pub sensitive_data_warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum Complexity {
    Beginner,
    Intermediate,
    Advanced,
}

impl std::fmt::Display for Complexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Complexity::Beginner => write!(f, "beginner"),
            Complexity::Intermediate => write!(f, "intermediate"),
            Complexity::Advanced => write!(f, "advanced"),
        }
    }
}

/// Trait for AI enrichment providers (local ONNX, cloud API, etc.)
pub trait EnrichmentProvider: Send + Sync {
    fn enrich(&self, snippet: &Snippet) -> Result<SnippetEnrichment>;
    fn detect_language(&self, content: &str) -> Result<String>;
    fn detect_sensitive_data(&self, content: &str) -> Result<Vec<String>>;
}

/// Rule-based enrichment that works offline without any AI model.
/// Provides basic language detection and sensitive data scanning.
pub struct RuleBasedEnrichment;

impl RuleBasedEnrichment {
    pub fn new() -> Self {
        Self
    }

    fn detect_language_from_content(content: &str) -> Option<String> {
        let indicators: &[(&str, &[&str])] = &[
            (
                "python",
                &[
                    "def ", "import ", "from ", "print(", "class ", "elif ", "self.",
                ],
            ),
            (
                "javascript",
                &["const ", "let ", "var ", "function ", "=> ", "console.log"],
            ),
            (
                "typescript",
                &[
                    "interface ",
                    "type ",
                    ": string",
                    ": number",
                    "as ",
                    "readonly ",
                ],
            ),
            (
                "rust",
                &[
                    "fn ", "let mut ", "impl ", "pub fn", "use ", "match ", "enum ", "&self",
                ],
            ),
            (
                "go",
                &["func ", "package ", "import (", "fmt.", "go func", ":= "],
            ),
            (
                "sql",
                &[
                    "SELECT ",
                    "INSERT ",
                    "CREATE TABLE",
                    "ALTER ",
                    "JOIN ",
                    "WHERE ",
                ],
            ),
            (
                "yaml",
                &["apiVersion:", "kind:", "metadata:", "spec:", "services:"],
            ),
            ("html", &["<!DOCTYPE", "<html", "<div", "<span", "<head"]),
            (
                "css",
                &["display:", "margin:", "padding:", "font-size:", "@media"],
            ),
            (
                "bash",
                &["#!/bin/bash", "echo ", "if [", "fi", "done", "export "],
            ),
            (
                "java",
                &[
                    "public class",
                    "System.out",
                    "private ",
                    "void ",
                    "new ",
                    "throws ",
                ],
            ),
        ];

        let mut best_match = None;
        let mut best_score = 0;

        for (lang, patterns) in indicators {
            let score = patterns.iter().filter(|p| content.contains(*p)).count();
            if score > best_score {
                best_score = score;
                best_match = Some(lang.to_string());
            }
        }

        if best_score >= 2 {
            best_match
        } else {
            None
        }
    }

    fn scan_sensitive_patterns(content: &str) -> Vec<String> {
        let mut warnings = Vec::new();

        let sensitive_patterns = [
            ("API key", &["api_key", "apikey", "api-key", "APIKEY"][..]),
            (
                "Secret/Token",
                &["secret", "token", "bearer", "authorization"],
            ),
            ("Password", &["password", "passwd", "pwd"]),
            (
                "Private key",
                &["PRIVATE KEY", "-----BEGIN RSA", "-----BEGIN EC"],
            ),
            ("AWS credentials", &["AKIA", "aws_access_key", "aws_secret"]),
            (
                "Connection string",
                &["mongodb://", "postgres://", "mysql://", "redis://"],
            ),
        ];

        let content_lower = content.to_lowercase();
        for (label, patterns) in &sensitive_patterns {
            for pattern in *patterns {
                if content_lower.contains(&pattern.to_lowercase()) {
                    warnings.push(format!("Possible {label} detected"));
                    break;
                }
            }
        }

        warnings
    }
}

impl Default for RuleBasedEnrichment {
    fn default() -> Self {
        Self::new()
    }
}

impl EnrichmentProvider for RuleBasedEnrichment {
    fn enrich(&self, snippet: &Snippet) -> Result<SnippetEnrichment> {
        let content = snippet.searchable_content();
        let warnings = Self::scan_sensitive_patterns(&content);

        let detected_language = snippet.fragments.first().and_then(|f| {
            if f.language == "plaintext" || f.language.is_empty() {
                Self::detect_language_from_content(&f.content)
            } else {
                None
            }
        });

        let line_count: usize = snippet
            .fragments
            .iter()
            .map(|f| f.content.lines().count())
            .sum();
        let complexity = if line_count < 10 {
            Complexity::Beginner
        } else if line_count < 50 {
            Complexity::Intermediate
        } else {
            Complexity::Advanced
        };

        Ok(SnippetEnrichment {
            suggested_title: None,
            suggested_description: None,
            suggested_tags: Vec::new(),
            detected_language,
            detected_tech_stack: None,
            complexity,
            has_sensitive_data: !warnings.is_empty(),
            sensitive_data_warnings: warnings,
        })
    }

    fn detect_language(&self, content: &str) -> Result<String> {
        Ok(Self::detect_language_from_content(content).unwrap_or_else(|| "plaintext".to_string()))
    }

    fn detect_sensitive_data(&self, content: &str) -> Result<Vec<String>> {
        Ok(Self::scan_sensitive_patterns(content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let enrichment = RuleBasedEnrichment::new();

        assert_eq!(
            enrichment
                .detect_language("def hello():\n    print('hi')\n    import os")
                .unwrap(),
            "python"
        );

        assert_eq!(
            enrichment
                .detect_language("const x = 1;\nlet y = 2;\nconsole.log(x + y)")
                .unwrap(),
            "javascript"
        );

        assert_eq!(
            enrichment
                .detect_language("fn main() {\n    let mut x = 5;\n    println!(\"{}\", x);\n}")
                .unwrap(),
            "rust"
        );
    }

    #[test]
    fn test_sensitive_data_detection() {
        let enrichment = RuleBasedEnrichment::new();

        let warnings = enrichment
            .detect_sensitive_data("const API_KEY = 'sk-1234567890'")
            .unwrap();
        assert!(!warnings.is_empty());

        let warnings = enrichment
            .detect_sensitive_data("const greeting = 'hello world'")
            .unwrap();
        assert!(warnings.is_empty());

        let warnings = enrichment
            .detect_sensitive_data("mongodb://user:pass@localhost:27017/db")
            .unwrap();
        assert!(!warnings.is_empty());
    }
}
