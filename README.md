# Cole

Cole is a local-first AI task assistant that turns checklist items into a simple Focus / Next / Finish flow.

The app is designed as a quiet desktop workspace, not a full project management dashboard. Cole starts with local Obsidian Markdown checklists, stores normalized tasks in SQLite, and uses AI only to summarize and arrange the work when the user allows it.

## Direction

- Local-first desktop app
- No central server for MVP
- Single-screen Visual Canvas
- Bottom chat composer
- Obsidian-first task ingestion
- SQLite as the local source of truth
- AI recommendation output validated before rendering
- No automatic write-back to source systems

## Planned Stack

- Tauri v2
- React 18
- TypeScript
- Vite
- Bun
- Tailwind CSS
- Zustand
- TanStack Query
- Rust backend commands
- SQLite with `rusqlite`

## MVP Scope

Cole MVP should:

1. Load checklist items from an Obsidian vault.
2. Normalize checklist items into local SQLite tasks.
3. Render a single-screen Visual Canvas.
4. Group tasks into Focus, Next, and Finish.
5. Provide a calm bottom chat composer.
6. Allow local done state.
7. Use an OpenAI-compatible model for validated recommendations when enabled.
8. Keep working without LLM access through deterministic fallback grouping.

## Development

This repository currently contains the product and agent operating contract. The Tauri app scaffold has not been created yet.

Expected commands after the scaffold exists:

```bash
bun install
bun run tauri dev
bun run lint
bun run typecheck
bun run test
cd src-tauri && cargo test
cd src-tauri && cargo fmt --check
cd src-tauri && cargo clippy --all-targets --all-features
bun run tauri build
```

## Project Contract

See [AGENTS.md](AGENTS.md) for architecture constraints, MVP scope, UI direction, implementation order, and AI-agent workspace rules.
