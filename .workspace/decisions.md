# Decisions

Durable decisions made by AI agents while building Cole.

## 2026-07-08

- Use `AGENTS.md` as the primary project and agent operating contract.
- Use a compact `.workspace/` structure with exactly four management files for now: `decisions.md`, `history.md`, `plan.md`, and `todo.md`.
- Keep `.workspace/` committed with the repository so future agents can resume from durable project state.
- Use Tauri v2, React 18, TypeScript, Vite, Rust backend commands, and SQLite as the fixed MVP stack.
- Use Bun as the frontend package manager, JavaScript runtime, and package script runner.
- Use `rusqlite` in the Rust backend as the preferred SQLite binding for task, source, sync, and recommendation logic.
- Keep the MVP focused on a single-screen Visual Canvas with a bottom Chat Composer.
- Start with Obsidian Markdown checklist parsing before adding Notion or Dooray.
- Preserve Cole's MVP constraints from `AGENTS.md`: local first, SQLite first, single screen first, Visual Canvas first, Obsidian first, LLM as assistant rather than authority, user confirmation before write-back, and no required central server.
