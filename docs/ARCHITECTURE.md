# Architecture

## Overview

SnipForge follows a **core library + thin clients** architecture. The Rust core (`snipforge-core`) contains all business logic, and each client (CLI, desktop, extensions) is a thin wrapper that calls into the core.

```
┌──────────────────────────────────────────────────────────┐
│                    CLIENT LAYER                          │
│                                                          │
│  ┌─────────┐ ┌──────┐ ┌─────────┐ ┌───┐ ┌───────────┐  │
│  │ Desktop │ │  IDE │ │ Browser │ │CLI│ │MCP Server │  │
│  │ (Tauri) │ │Plugin│ │  Ext.   │ │   │ │           │  │
│  └────┬────┘ └──┬───┘ └────┬────┘ └─┬─┘ └─────┬─────┘  │
│       │         │          │        │          │         │
└───────┼─────────┼──────────┼────────┼──────────┼─────────┘
        │         │          │        │          │
        └─────────┴──────┬───┴────────┴──────────┘
                         │
                    ┌────▼─────┐
                    │ Core SDK │  (Rust library / WASM)
                    │          │
                    │ - CRUD   │
                    │ - Search │
                    │ - Sync   │
                    │ - AI     │
                    └────┬─────┘
                         │
              ┌──────────┼──────────┐
              │          │          │
        ┌─────▼───┐ ┌───▼────┐ ┌──▼──────────┐
        │ Local   │ │ Vector │ │ Sync Engine │
        │ Storage │ │ Index  │ │             │
        │ SQLite  │ │LanceDB │ │  CRDTs /    │
        │         │ │        │ │  Yjs-based  │
        └─────────┘ └────────┘ └──────┬──────┘
                                      │
                              ┌───────▼───────┐
                              │  Cloud Layer  │
                              │  (Optional)   │
                              │               │
                              │ - Team sync   │
                              │ - Auth/SSO    │
                              │ - Analytics   │
                              │ - Marketplace │
                              └───────────────┘
```

## Core Library (`crates/core`)

### Modules

- **`models/`** — Data structures: `Snippet`, `Fragment`, `TechStack`, filter/param types
- **`storage/`** — Storage trait + SQLite implementation with FTS5 full-text search
- **`search/`** — Search engine trait + keyword search; semantic search planned
- **`ai/`** — Enrichment trait + rule-based language detection, sensitive data scanning

### Key Design Decisions

1. **Trait-based backends** — Storage and Search are defined as traits, allowing swappable backends (SQLite, PostgreSQL, in-memory for tests)
2. **SQLite FTS5** — Full-text search integrated directly into the storage layer for keyword search
3. **Local-first** — All data stored locally by default; cloud sync is optional
4. **Thread-safe** — All stores use `Mutex<Connection>` for safe concurrent access

## CLI (`crates/cli`)

Thin wrapper using `clap` for argument parsing. Delegates all logic to `snipforge-core`.

## Desktop App (`desktop/`)

- **Backend**: Tauri 2.0 (Rust) — wraps `snipforge-core` and exposes Tauri commands
- **Frontend**: React + TypeScript + Vite — lightweight, fast UI

## VS Code Extension (`extensions/vscode/`)

TypeScript extension that will invoke the CLI or a local SDK for snippet operations.

## Future: Semantic Search

The planned architecture for AI-powered search:

1. **Embedding Model**: CodeBERT/UniXcoder via ONNX Runtime (on-device)
2. **Vector Storage**: LanceDB (embedded, no separate process)
3. **Hybrid Ranking**: `score = α × vector_similarity + β × BM25_keyword_score`
4. **Index**: Automatically index snippets on save; lazy reindex on startup

## Future: Team Sync

- **Protocol**: CRDTs via Yrs (Rust port of Yjs) for conflict-free merging
- **Transport**: WebSocket for real-time; REST API for batch operations
- **Auth**: OIDC/SAML via WorkOS for enterprise SSO
