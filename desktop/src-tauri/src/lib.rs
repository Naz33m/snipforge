use serde::{Deserialize, Serialize};
use snipforge_core::models::*;
use snipforge_core::SnipForge;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub engine: Mutex<SnipForge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnippetResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub fragments: Vec<FragmentResponse>,
    pub tags: Vec<String>,
    pub tech_stack: Option<String>,
    pub folder: Option<String>,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FragmentResponse {
    pub label: String,
    pub language: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultResponse {
    pub snippet: SnippetResponse,
    pub score: f64,
    pub match_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total: usize,
    pub favorites: usize,
    pub by_language: Vec<(String, usize)>,
    pub by_tech_stack: Vec<(String, usize)>,
}

fn snippet_to_response(s: &Snippet) -> SnippetResponse {
    SnippetResponse {
        id: s.id.clone(),
        title: s.title.clone(),
        description: s.description.clone(),
        fragments: s
            .fragments
            .iter()
            .map(|f| FragmentResponse {
                label: f.label.clone(),
                language: f.language.clone(),
                content: f.content.clone(),
            })
            .collect(),
        tags: s.tags.clone(),
        tech_stack: s.tech_stack.as_ref().map(|ts| ts.to_string()),
        folder: s.folder.clone(),
        is_favorite: s.is_favorite,
        created_at: s.created_at.to_rfc3339(),
        updated_at: s.updated_at.to_rfc3339(),
    }
}

#[tauri::command]
fn list_snippets(
    state: State<AppState>,
    tech_stack: Option<String>,
    language: Option<String>,
    tag: Option<String>,
    favorites_only: Option<bool>,
) -> Result<Vec<SnippetResponse>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let filter = SnippetFilter {
        tech_stack: tech_stack.map(|s| TechStack::parse(&s)),
        language,
        tags: tag.map(|t| vec![t]),
        favorites_only: favorites_only.unwrap_or(false),
        ..Default::default()
    };
    let snippets = engine.list_snippets(filter).map_err(|e| e.to_string())?;
    Ok(snippets.iter().map(snippet_to_response).collect())
}

#[tauri::command]
fn get_snippet(state: State<AppState>, id: String) -> Result<SnippetResponse, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let snippet = engine.get_snippet(&id).map_err(|e| e.to_string())?;
    Ok(snippet_to_response(&snippet))
}

#[derive(Debug, Deserialize)]
pub struct CreateSnippetArgs {
    pub title: String,
    pub language: String,
    pub content: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub tech_stack: Option<String>,
    pub folder: Option<String>,
}

#[tauri::command]
fn create_snippet(
    state: State<AppState>,
    args: CreateSnippetArgs,
) -> Result<SnippetResponse, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let params = CreateSnippetParams {
        title: args.title,
        description: args.description.unwrap_or_default(),
        language: args.language,
        content: args.content,
        tags: args.tags.unwrap_or_default(),
        tech_stack: args.tech_stack.map(|s| TechStack::parse(&s)),
        folder: args.folder,
    };
    let snippet = engine.create_snippet(params).map_err(|e| e.to_string())?;
    Ok(snippet_to_response(&snippet))
}

#[tauri::command]
fn update_snippet(
    state: State<AppState>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    tech_stack: Option<String>,
    is_favorite: Option<bool>,
) -> Result<SnippetResponse, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let params = UpdateSnippetParams {
        title,
        description,
        tags,
        tech_stack: tech_stack.map(|s| TechStack::parse(&s)),
        is_favorite,
        ..Default::default()
    };
    let snippet = engine.update_snippet(&id, params).map_err(|e| e.to_string())?;
    Ok(snippet_to_response(&snippet))
}

#[tauri::command]
fn delete_snippet(state: State<AppState>, id: String) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.delete_snippet(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn search_snippets(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SearchResultResponse>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let results = engine
        .search(&query, limit.unwrap_or(20))
        .map_err(|e| e.to_string())?;
    Ok(results
        .iter()
        .map(|r| SearchResultResponse {
            snippet: snippet_to_response(&r.snippet),
            score: r.score,
            match_reason: r.match_reason.to_string(),
        })
        .collect())
}

#[tauri::command]
fn toggle_favorite(state: State<AppState>, id: String) -> Result<SnippetResponse, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let snippet = engine.get_snippet(&id).map_err(|e| e.to_string())?;
    let params = UpdateSnippetParams {
        is_favorite: Some(!snippet.is_favorite),
        ..Default::default()
    };
    let updated = engine.update_snippet(&id, params).map_err(|e| e.to_string())?;
    Ok(snippet_to_response(&updated))
}

#[tauri::command]
fn get_stats(state: State<AppState>) -> Result<StatsResponse, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let snippets = engine
        .list_snippets(SnippetFilter::default())
        .map_err(|e| e.to_string())?;

    let total = snippets.len();
    let favorites = snippets.iter().filter(|s| s.is_favorite).count();

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

    let mut by_language: Vec<_> = lang_counts.into_iter().collect();
    by_language.sort_by_key(|item| std::cmp::Reverse(item.1));

    let mut by_tech_stack: Vec<_> = stack_counts.into_iter().collect();
    by_tech_stack.sort_by_key(|item| std::cmp::Reverse(item.1));

    Ok(StatsResponse {
        total,
        favorites,
        by_language,
        by_tech_stack,
    })
}

fn get_data_dir() -> PathBuf {
    dirs_next::data_dir()
        .map(|d| d.join("snipforge"))
        .unwrap_or_else(|| PathBuf::from(".snipforge"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir = get_data_dir();
    let engine = SnipForge::new(data_dir).expect("Failed to initialize SnipForge engine");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            engine: Mutex::new(engine),
        })
        .invoke_handler(tauri::generate_handler![
            list_snippets,
            get_snippet,
            create_snippet,
            update_snippet,
            delete_snippet,
            search_snippets,
            toggle_favorite,
            get_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
