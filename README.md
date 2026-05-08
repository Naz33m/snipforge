# SnipForge

**Advanced, cross-platform code snippet manager with AI-powered search and team sharing.**

SnipForge is designed for developers who want their snippet library organized by tech stack, searchable by meaning (not just keywords), and shareable with their team — all while keeping data local-first and privacy-respecting.

## Features

### Core
- **Multi-fragment snippets** — group related files/tabs in a single snippet
- **500+ language support** — syntax highlighting via Tree-sitter grammars
- **Nested folders + tags** — organize snippets however you prefer
- **Full-text search** — powered by SQLite FTS5
- **Template variables** — parameterized snippets with `{{placeholder}}` support
- **Import/Export** — JSON format, compatible with other tools

### AI-Powered (Roadmap)
- **Semantic search** — find snippets by meaning using on-device embeddings
- **Auto-enrichment** — AI generates descriptions, tags, and language detection
- **Sensitive data detection** — flags API keys, passwords, and PII
- **Code transformation** — translate between languages, optimize for readability

### Tech-Stack Collections
Snippets organized by technology ecosystem:
- React, Node.js, Python, Rust, Go, SQL
- DevOps, AI/ML, Mobile, Data Engineering
- Curated starter collections per stack

### Team Sharing (Roadmap)
- Shared workspaces with role-based access
- Snippet reviews and comments
- Usage analytics
- Real-time sync via CRDTs

### Cross-Platform
- **Desktop app** (Tauri 2.0 + React) — fully wired to core engine
- **CLI tool** — full-featured terminal interface
- VS Code extension (roadmap)
- JetBrains plugin (roadmap)
- Browser extension (roadmap)
- MCP Server for AI assistants (roadmap)

## Quick Start

### Install from source

```bash
# Clone the repository
git clone https://github.com/Naz33m/snipforge.git
cd snipforge

# Build
cargo build --release

# The CLI binary will be at target/release/snipforge
```

### CLI Usage

```bash
# Add a snippet
snipforge add --title "React useState" --language typescript \
  --content "const [state, setState] = useState(initialValue)" \
  --tags "hooks,react,state" --stack react

# Search snippets
snipforge search "react hooks"

# List all snippets
snipforge list

# Filter by tech stack
snipforge list --stack python

# Show a specific snippet
snipforge show <snippet-id>

# Export all snippets
snipforge export --output my-snippets.json

# Import snippets
snipforge import my-snippets.json

# View statistics
snipforge stats
```

## Desktop App

The desktop app provides a full GUI for managing snippets, powered by [Tauri 2.0](https://tauri.app/) + React.

### Prerequisites

- **Rust 1.85+** (install via [rustup.rs](https://rustup.rs/))
- **Node.js 18+** (install via [nvm](https://github.com/nvm-sh/nvm) or [nodejs.org](https://nodejs.org/))
- **System dependencies** (Linux only):
  ```bash
  sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
  ```

### Run the Desktop App

```bash
cd desktop
npm install
npx tauri dev
```

This will build the Rust backend and launch the desktop window with the React UI connected to the full snipforge-core engine.

### Features
- **Add snippets** — form with title, language, code, description, tags, tech stack
- **Search** — full-text search powered by SQLite FTS5
- **Browse** — filter by tech stack, view all snippets
- **Detail view** — see all fragments, metadata, and tags
- **Favorite** — toggle favorite status
- **Delete** — remove snippets

## Architecture

```
snipforge/
├── crates/
│   ├── core/          # Rust core library (models, storage, search, AI)
│   └── cli/           # CLI application
├── desktop/           # Tauri 2.0 desktop app
│   ├── src-tauri/     # Rust backend (Tauri commands)
│   └── src/           # React frontend
├── extensions/
│   └── vscode/        # VS Code extension (scaffold)
└── docs/              # Documentation
```

### Tech Stack

| Layer | Technology |
|---|---|
| Core engine | Rust |
| Local storage | SQLite (WAL mode) + FTS5 |
| Search | Keyword (FTS5) + Semantic (planned: LanceDB + CodeBERT) |
| Desktop | Tauri 2.0 + React |
| CLI | Rust + clap |
| Sync | CRDTs via Yrs (planned) |

## Development

```bash
# Run tests (core + CLI)
cargo test

# Run with logging
RUST_LOG=debug cargo run -- stats

# Run clippy
cargo clippy --all-targets

# Format code
cargo fmt

# Build desktop Rust backend (from desktop/src-tauri/)
cd desktop/src-tauri && cargo build

# Build desktop frontend (from desktop/)
cd desktop && npm run build
```

## Roadmap

- [x] Core Rust engine with CRUD operations
- [x] SQLite storage with FTS5 full-text search
- [x] Keyword search engine
- [x] CLI tool with add, search, list, show, delete, export, import
- [x] Rule-based language detection and sensitive data scanning
- [ ] On-device semantic search (LanceDB + CodeBERT embeddings)
- [x] Tauri desktop application (wired to core engine)
- [ ] VS Code extension
- [ ] Team workspaces and cloud sync
- [ ] MCP Server for AI assistant integration
- [ ] Curated tech-stack starter collections
- [ ] Snippet marketplace

## License

MIT — see [LICENSE](LICENSE) for details.
