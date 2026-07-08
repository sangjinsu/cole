# History

Dated work history, verification notes, and major project milestones.

## 2026-07-08

- Read `AGENTS.md` and treated it as the governing contract for Cole.
- Prepared the project management workspace requested by the user.
- Added `.workspace/` as durable AI project state for decisions, history, plans, and todos.
- Added `AGENTS.md` instructions requiring agents to read and maintain `.workspace/` during substantial work.
- Rewrote `AGENTS.md` around the Tauri v2 + React + TypeScript + Rust + SQLite direction requested by the user.
- Updated the frontend tooling direction to use Bun for dependency installation and package scripts.
- Reduced the MVP contract to Obsidian-first task ingestion, a single-screen Visual Canvas, Focus / Next / Finish recommendations, a bottom Chat Composer, local done state, and validated AI recommendation output.
- Prepared the initial `README.md` and GitHub publication target for `sangjinsu/cole`.
- Created the public GitHub repository `sangjinsu/cole` at `https://github.com/sangjinsu/cole`.
- Pushed `main` to `origin/main` with initial project contract docs and README.
- Verified the GitHub repository description: `Local-first AI task assistant that turns checklist items into a Focus / Next / Finish flow.`
- Updated the implementation direction to include OpenUI for VisualCanvas rendering only, keeping storage, sync, parser, and source connector logic outside OpenUI.
- Created branch `agent/obsidian-mvp` for product implementation.
- Scaffolded the Tauri v2 + React 18 + TypeScript + Vite app using Bun.
- Added Tailwind CSS, Zustand, TanStack Query, OpenUI React Lang, Vitest, ESLint, and Prettier.
- Implemented Rust `rusqlite` local storage, source/task DTOs, Tauri commands, Obsidian Markdown checklist parsing, Focus / Next / Finish recommendations, and local done state.
- Implemented the single-screen Cole UI with `AppShell`, OpenUI-backed `VisualCanvas`, deterministic fallback rendering, `TaskFlowCard`, `SourceBadge`, `EmptyState`, and fixed bottom `ChatComposer`.
- Fixed review blockers by routing default OpenUI `TaskCard` mark-done actions through `Renderer.onAction` and extending SQLite `tasks` schema/migrations to include the minimum local task fields required by `AGENTS.md`.
- Verified with `bun run test`, `cd src-tauri && cargo test`, `bun run typecheck`, `bun run lint`, `cd src-tauri && cargo fmt --check`, `cd src-tauri && cargo clippy --all-targets --all-features`, `bun run build`, and `bun run tauri build`.
