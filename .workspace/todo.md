# Todo

Actionable project tasks for AI agents. Use Markdown checkboxes and update this file before and after substantial work.

## Project Management

- [x] Read `AGENTS.md` before starting implementation.
- [x] Create `.workspace/` as durable AI project state.
- [x] Add `.workspace/decisions.md`.
- [x] Add `.workspace/history.md`.
- [x] Add `.workspace/plan.md`.
- [x] Add `.workspace/todo.md`.
- [x] Document `.workspace/` management rules in `AGENTS.md`.
- [x] Rewrite `AGENTS.md` around Tauri, Rust, single-screen Visual Canvas, and Obsidian-first MVP direction.
- [x] Set Bun as the frontend package manager and script runner in `AGENTS.md`.
- [x] Prepare initial README and GitHub repository metadata.
- [x] Create public GitHub repository `sangjinsu/cole`.
- [x] Push `main` to `origin/main`.
- [x] Verify README and repository description on GitHub.
- [x] Add OpenUI VisualCanvas-only direction to project contract.
- [x] Start Phase 0 Tauri product skeleton when requested.

## Phase 0: Tauri Project Skeleton

- [x] Create Tauri + React + TypeScript project using Bun.
- [x] Build `AppShell`, OpenUI-backed `VisualCanvas`, and normal React `ChatComposer`.
- [x] Add SQLite local database.
- [x] Implement local task model.
- [x] Implement Obsidian Markdown checklist parser.
- [x] Render tasks as Focus / Next / Finish cards through registered OpenUI components.
- [x] Add deterministic React fallback rendering for invalid OpenUI output.
- [x] Route OpenUI `TaskCard` mark-done actions to local task completion.
- [x] Verify SQLite `tasks` schema contains the minimum local task fields from `AGENTS.md`.
- [ ] Add OpenAI-compatible LLM adapter.
- [ ] Add AI recommendation schema validation.
- [x] Add local done state.

## Verification

- [x] `bun run test`
- [x] `cd src-tauri && cargo test`
- [x] `bun run typecheck`
- [x] `bun run lint`
- [x] `cd src-tauri && cargo fmt --check`
- [x] `cd src-tauri && cargo clippy --all-targets --all-features`
- [x] `bun run build`
- [x] `bun run tauri build`
