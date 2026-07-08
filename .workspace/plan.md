# Plan

Current implementation plan and active project phase.

## Active Phase

Phase 0: Tauri project skeleton and single-screen MVP foundation is implemented on branch `agent/obsidian-mvp`.

## Current Management Setup

1. Keep `AGENTS.md` as the product, architecture, and agent workflow source of truth.
2. Keep `.workspace/decisions.md`, `.workspace/history.md`, `.workspace/plan.md`, and `.workspace/todo.md` updated as durable project state.
3. Before product implementation starts, confirm the management workspace is present and referenced from `AGENTS.md`.

## Completed in Current Implementation

1. Created a Tauri + React + TypeScript project using Bun.
2. Built a single-screen UI with `AppShell`, OpenUI-backed `VisualCanvas`, and normal React `ChatComposer`.
3. Added a local SQLite database through Rust `rusqlite`.
4. Implemented the local task/source DTO model.
5. Implemented an Obsidian Markdown checklist parser.
6. Rendered tasks as Focus / Next / Finish cards through registered OpenUI components when valid.
7. Added deterministic React fallback rendering for invalid OpenUI output.
8. Added local done state.
9. Added OpenUI render-path mark-done action routing.
10. Added SQLite schema coverage for the minimum local task fields.
11. Added frontend and Rust tests for the core parser/recommendation/rendering path.

## Next Product Implementation Direction

Follow `AGENTS.md` implementation order unless the user changes priority:

1. Add OpenAI-compatible LLM adapter.
2. Add AI recommendation schema validation.
3. Add recommendation cache persistence.
4. Add cautious Obsidian write-back preview.
5. Add Notion connector.
6. Add Dooray connector.
