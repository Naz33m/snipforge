use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use snipforge_core::models::*;
use snipforge_core::SnipForge;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "snipforge",
    about = "SnipForge — AI-powered code snippet manager",
    version,
    author
)]
struct Cli {
    /// Data directory for snippet storage
    #[arg(long, env = "SNIPFORGE_DATA_DIR")]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new snippet
    Add {
        /// Snippet title
        #[arg(short, long)]
        title: String,

        /// Programming language
        #[arg(short, long)]
        language: String,

        /// Code content (reads from stdin if not provided)
        #[arg(short, long)]
        content: Option<String>,

        /// Description
        #[arg(short, long, default_value = "")]
        description: String,

        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Tech stack (e.g., react, python, nodejs, devops)
        #[arg(long)]
        stack: Option<String>,

        /// Folder path
        #[arg(long)]
        folder: Option<String>,
    },

    /// Search snippets
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// List all snippets
    List {
        /// Filter by tech stack
        #[arg(long)]
        stack: Option<String>,

        /// Filter by language
        #[arg(long)]
        language: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Show favorites only
        #[arg(long)]
        favorites: bool,
    },

    /// Show a specific snippet
    Show {
        /// Snippet ID
        id: String,
    },

    /// Delete a snippet
    Delete {
        /// Snippet ID
        id: String,
    },

    /// Toggle favorite status
    Favorite {
        /// Snippet ID
        id: String,
    },

    /// Export snippets to JSON
    Export {
        /// Output file path (stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import snippets from JSON
    Import {
        /// Input file path
        file: PathBuf,
    },

    /// Show statistics
    Stats,
}

fn get_data_dir(cli_dir: Option<PathBuf>) -> PathBuf {
    cli_dir.unwrap_or_else(|| {
        dirs_next::data_dir()
            .map(|d| d.join("snipforge"))
            .unwrap_or_else(|| PathBuf::from(".snipforge"))
    })
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let data_dir = get_data_dir(cli.data_dir);
    let engine = SnipForge::new(data_dir)?;

    match cli.command {
        Commands::Add {
            title,
            language,
            content,
            description,
            tags,
            stack,
            folder,
        } => {
            let content = match content {
                Some(c) => c,
                None => {
                    let mut buf = String::new();
                    eprintln!("{}", "Reading code from stdin (Ctrl+D to finish):".dimmed());
                    std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
                    buf
                }
            };

            let tags = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let tech_stack = stack.map(|s| TechStack::parse(&s));

            let snippet = engine.create_snippet(CreateSnippetParams {
                title,
                description,
                language,
                content,
                tags,
                tech_stack,
                folder,
            })?;

            println!("{} Snippet created: {}", "✓".green().bold(), snippet.id);
            println!("  Title: {}", snippet.title.bold());
        }

        Commands::Search { query, limit } => {
            let results = engine.search(&query, limit)?;

            if results.is_empty() {
                println!("{}", "No snippets found.".dimmed());
                return Ok(());
            }

            println!(
                "{} {} result(s) for \"{}\":\n",
                "Found".green().bold(),
                results.len(),
                query.cyan()
            );

            for (i, result) in results.iter().enumerate() {
                let s = &result.snippet;
                println!(
                    "  {}. {} {}",
                    (i + 1).to_string().dimmed(),
                    s.title.bold(),
                    format!("[{}]", s.primary_language()).cyan()
                );
                println!("     ID: {}", s.id.dimmed());
                if !s.description.is_empty() {
                    println!("     {}", s.description.dimmed());
                }
                if !s.tags.is_empty() {
                    println!("     Tags: {}", s.tags.join(", ").yellow());
                }
                println!(
                    "     Score: {:.1} ({})",
                    result.score,
                    result.match_reason.to_string().dimmed()
                );
                println!();
            }
        }

        Commands::List {
            stack,
            language,
            tag,
            favorites,
        } => {
            let filter = SnippetFilter {
                tech_stack: stack.map(|s| TechStack::parse(&s)),
                language,
                tags: tag.map(|t| vec![t]),
                favorites_only: favorites,
                ..Default::default()
            };

            let snippets = engine.list_snippets(filter)?;

            if snippets.is_empty() {
                println!("{}", "No snippets found.".dimmed());
                return Ok(());
            }

            println!(
                "{} snippet(s):\n",
                snippets.len().to_string().green().bold()
            );

            for s in &snippets {
                let fav = if s.is_favorite { "★ " } else { "" };
                let stack_label = s
                    .tech_stack
                    .as_ref()
                    .map(|ts| format!(" ({})", ts))
                    .unwrap_or_default();

                println!(
                    "  {}{} {} {}",
                    fav.yellow(),
                    s.title.bold(),
                    format!("[{}]", s.primary_language()).cyan(),
                    stack_label.dimmed()
                );
                println!("  ID: {}", s.id.dimmed());
                if !s.tags.is_empty() {
                    println!("  Tags: {}", s.tags.join(", ").yellow());
                }
                println!();
            }
        }

        Commands::Show { id } => {
            let snippet = engine.get_snippet(&id)?;

            println!("{}", snippet.title.bold());
            println!("{}", "─".repeat(50).dimmed());

            if !snippet.description.is_empty() {
                println!("{}", snippet.description);
                println!();
            }

            for fragment in &snippet.fragments {
                if snippet.fragments.len() > 1 {
                    println!("── {} ──", fragment.label.cyan().bold());
                }
                println!("{}", format!("Language: {}", fragment.language).dimmed());
                println!();
                println!("{}", fragment.content);
                println!();
            }

            println!("{}", "─".repeat(50).dimmed());
            println!("ID: {}", snippet.id.dimmed());
            if let Some(ref stack) = snippet.tech_stack {
                println!("Stack: {stack}");
            }
            if !snippet.tags.is_empty() {
                println!("Tags: {}", snippet.tags.join(", ").yellow());
            }
            if snippet.is_favorite {
                println!("{}", "★ Favorite".yellow());
            }
            println!("Created: {}", snippet.created_at.format("%Y-%m-%d %H:%M"));
            println!("Updated: {}", snippet.updated_at.format("%Y-%m-%d %H:%M"));
        }

        Commands::Delete { id } => {
            let snippet = engine.get_snippet(&id)?;
            engine.delete_snippet(&id)?;
            println!("{} Deleted: {}", "✓".green().bold(), snippet.title.bold());
        }

        Commands::Favorite { id } => {
            let snippet = engine.get_snippet(&id)?;
            let new_fav = !snippet.is_favorite;
            engine.update_snippet(
                &id,
                UpdateSnippetParams {
                    is_favorite: Some(new_fav),
                    ..Default::default()
                },
            )?;
            let status = if new_fav { "favorited" } else { "unfavorited" };
            println!("{} {} {}", "✓".green().bold(), snippet.title.bold(), status);
        }

        Commands::Export { output } => {
            let json = engine.export_json()?;
            match output {
                Some(path) => {
                    std::fs::write(&path, &json)?;
                    println!(
                        "{} Exported to {}",
                        "✓".green().bold(),
                        path.display().to_string().cyan()
                    );
                }
                None => print!("{json}"),
            }
        }

        Commands::Import { file } => {
            let json = std::fs::read_to_string(&file)?;
            let count = engine.import_json(&json)?;
            println!(
                "{} Imported {} snippet(s)",
                "✓".green().bold(),
                count.to_string().cyan()
            );
        }

        Commands::Stats => {
            let count = engine.count()?;
            let snippets = engine.list_snippets(SnippetFilter::default())?;

            println!("{}", "SnipForge Statistics".bold());
            println!("{}", "─".repeat(30).dimmed());
            println!("Total snippets: {}", count.to_string().cyan().bold());

            let mut lang_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            let mut stack_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            for s in &snippets {
                *lang_counts
                    .entry(s.primary_language().to_string())
                    .or_default() += 1;
                if let Some(ref stack) = s.tech_stack {
                    *stack_counts.entry(stack.to_string()).or_default() += 1;
                }
            }

            if !lang_counts.is_empty() {
                println!("\nBy language:");
                let mut sorted: Vec<_> = lang_counts.into_iter().collect();
                sorted.sort_by_key(|item| std::cmp::Reverse(item.1));
                for (lang, count) in sorted {
                    println!("  {}: {}", lang.cyan(), count);
                }
            }

            if !stack_counts.is_empty() {
                println!("\nBy tech stack:");
                let mut sorted: Vec<_> = stack_counts.into_iter().collect();
                sorted.sort_by_key(|item| std::cmp::Reverse(item.1));
                for (stack, count) in sorted {
                    println!("  {}: {}", stack.cyan(), count);
                }
            }

            let favorites = snippets.iter().filter(|s| s.is_favorite).count();
            println!("\nFavorites: {}", favorites.to_string().yellow());
            println!(
                "Data dir: {}",
                engine.data_dir().display().to_string().dimmed()
            );
        }
    }

    Ok(())
}
