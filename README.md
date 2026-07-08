# Cole

Cole is a local-first AI task assistant that turns checklist items into a simple Focus / Next / Finish flow.

The app is designed as a quiet desktop workspace, not a full project management dashboard. Cole starts with local Obsidian Markdown checklists, stores normalized tasks in SQLite, and uses AI only to summarize and arrange the work when the user allows it.

## Direction

- Local-first desktop app
- No central server for MVP
- Single-screen Visual Canvas
- OpenUI-style generative UI for the Visual Canvas only
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
- OpenUI for AI-generated VisualCanvas rendering
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
5. Compose the Visual Canvas from registered OpenUI components when possible.
6. Provide a calm bottom chat composer.
7. Allow local done state.
8. Use an OpenAI-compatible model for validated recommendations when enabled.
9. Keep working without LLM access through deterministic fallback grouping.

## Current Implementation

The repository now contains the first Tauri desktop implementation:

- Tauri v2 shell with Rust commands
- React 18 + TypeScript + Vite frontend run with Bun
- Tailwind CSS visual system
- OpenUI component library for `VisualCanvas`
- Deterministic React fallback when OpenUI Lang is invalid
- SQLite local database through `rusqlite`
- Obsidian Markdown checklist parser
- Focus / Next / Finish recommendation flow
- Local-only mark-done action

## Development

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
