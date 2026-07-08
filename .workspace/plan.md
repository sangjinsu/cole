# Plan

Current implementation plan and active project phase.

## Active Phase

Phase 0: Tauri project skeleton and single-screen MVP foundation.

## Current Management Setup

1. Keep `AGENTS.md` as the product, architecture, and agent workflow source of truth.
2. Keep `.workspace/decisions.md`, `.workspace/history.md`, `.workspace/plan.md`, and `.workspace/todo.md` updated as durable project state.
3. Before product implementation starts, confirm the management workspace is present and referenced from `AGENTS.md`.

## Next Product Implementation Direction

Follow `AGENTS.md` section 20 unless the user changes priority:

1. Create Tauri + React + TypeScript project using Bun.
2. Build the single-screen UI with `AppShell`, `VisualCanvas`, and `ChatComposer`.
3. Add SQLite local database.
4. Implement local task model.
5. Implement Obsidian Markdown checklist parser.
6. Render tasks as Focus / Next / Finish cards.
7. Add OpenAI-compatible LLM adapter.
8. Add AI recommendation schema validation.
9. Add local done state.
10. Add Notion connector.
11. Add Dooray connector.
