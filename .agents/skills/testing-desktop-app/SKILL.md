---
name: testing-snipforge-desktop
description: Test the SnipForge Tauri desktop app end-to-end. Use when verifying desktop GUI, CRUD operations, search, or CLI-desktop integration.
---

# Testing SnipForge Desktop App

## Prerequisites

- X11 display available (`:0` on Linux VMs)
- System packages: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf wmctrl`
- Node.js 18+ and Rust 1.85+
- Frontend dependencies installed: `cd desktop && npm install`

## Launch

```bash
# Clean slate (remove existing data for fresh test)
rm -rf ~/.local/share/snipforge

# Launch in background
cd /path/to/snipforge/desktop && npx tauri dev
```

First launch compiles the Rust backend (~10-30s). Subsequent launches are faster (~2s).

The app window opens automatically. Use `wmctrl -r "SnipForge" -b add,maximized_vert,maximized_horz` to maximize it for recording.

## Key Test Flows

1. **Empty state**: On first launch with no data, shows "No snippets yet" and "0 snippets · 0 ★"
2. **Create snippet**: Click "+ New Snippet" → fill form (title, language, tech stack dropdown, description, code textarea, tags) → "Save Snippet"
3. **Search**: Type in search bar + press Enter. Shows "Found N results for ..." banner with "Clear" link
4. **Detail view**: Click any snippet card → shows title, description, tech stack badge, date, tags, fragments with language badges, and snippet ID
5. **Favorite toggle**: In detail view, click "☆ Favorite" → changes to "★ Favorited", stats badge updates
6. **Delete**: In detail view, click "Delete" → returns to list
7. **CLI integration**: Snippets added via CLI appear in desktop list after navigating (click "SnipForge" title or change filter). Search won't find externally-added snippets because it uses an in-memory index built at startup.

## Known Behaviors

- **Search index is per-process**: The FTS5 search index is loaded into memory at app startup. External changes (CLI) won't be searchable until the app is restarted. `list_snippets` queries SQLite directly so new data appears in the list.
- **Stats badge staleness**: The stats badge only refreshes after create/delete/favorite operations, not when the list is reloaded via filter change.
- **Tech Stack dropdown**: Shows None, React, Node.js, Python, Rust, Go, SQL, DevOps, AI/ML, Mobile, Git, Data Engineering.
- **Filter dropdown on list view**: Changing the filter triggers a fresh SQLite query.

## UI Navigation Map

- **Header**: "SnipForge" title (click to reset to list), stats badge, "+ New Snippet" button
- **List view**: Search input + Search button + tech stack filter dropdown + snippet cards
- **Add view**: "← Back" link + form fields + "Save Snippet" / "Cancel" buttons
- **Detail view**: "← Back" link + "☆ Favorite" / "★ Favorited" button + "Delete" button + metadata + fragments

## Devin Secrets Needed

None — the desktop app runs fully locally with no external services.
