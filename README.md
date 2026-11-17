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

### Cross-Platform (Roadmap)
- Desktop app (Tauri 2.0)
- VS Code extension
- JetBrains plugin
- CLI tool
- Browser extension
- MCP Server for AI assistants

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

## Architecture

```
snipforge/
├── crates/
│   ├── core/          # Rust core library (models, storage, search, AI)
│   └── cli/           # CLI application
├── desktop/           # Tauri desktop app (planned)
│   ├── src-tauri/     # Rust backend
│   └── src/           # React frontend
├── extensions/
│   └── vscode/        # VS Code extension (planned)
└── docs/              # Documentation
```

### Tech Stack

| Layer | Technology |
|---|---|
| Core engine | Rust |
| Local storage | SQLite (WAL mode) + FTS5 |
| Search | Keyword (FTS5) + Semantic (planned: LanceDB + CodeBERT) |
| Desktop | Tauri 2.0 + React (planned) |
| CLI | Rust + clap |
| Sync | CRDTs via Yrs (planned) |

## Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- stats

# Run clippy
cargo clippy --all-targets

# Format code
cargo fmt
```

## Roadmap

- [x] Core Rust engine with CRUD operations
- [x] SQLite storage with FTS5 full-text search
- [x] Keyword search engine
- [x] CLI tool with add, search, list, show, delete, export, import
- [x] Rule-based language detection and sensitive data scanning
- [ ] On-device semantic search (LanceDB + CodeBERT embeddings)
- [ ] Tauri desktop application
- [ ] VS Code extension
- [ ] Team workspaces and cloud sync
- [ ] MCP Server for AI assistant integration
- [ ] Curated tech-stack starter collections
- [ ] Snippet marketplace

## Credits

Built with help from Devin AI.

## License

MIT — see [LICENSE](LICENSE) for details.
