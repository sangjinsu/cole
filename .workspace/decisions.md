# Decisions

Durable product and engineering decisions for Cole.

## 2026-07-08

- Use `AGENTS.md` as the primary product and agent operating contract.
- Keep exactly four committed workspace files: `decisions.md`, `history.md`, `plan.md`, and `todo.md`.
- Use Tauri v2, React 18, TypeScript, Vite, Bun, Rust commands, and SQLite through `rusqlite`.
- Store the default SQLite database under the OS user-data directory, with `COLE_DATA_DIR` as a development/test override.
- Keep Cole local-first with no required central server.

## 2026-07-11

- Replace the Obsidian-first prototype contract with one fixed default local SQLite checklist for the MVP.
- Make Checklist View the default source-of-truth workspace and Analysis View a secondary read-only interpretation.
- Model the checklist as arbitrary-depth `task` and `group` nodes. Provide inline create, rename, check, estimate, collapse, and archive controls; exclude reorder and reparent.
- Maintain each task as a 1:1 projection of its checklist node and mutate both records in one transaction.
- Increment checklist revision once per semantic mutation and use a canonical checklist hash for analysis cache and stale detection.
- Reject parent-task completion while incomplete descendants remain. Prevent child creation below a completed task and descendant uncheck below a completed task ancestor.
- Treat delete as archive. Require explicit cascade confirmation before archiving a node with descendants.
- Use OpenUI only in Analysis View. Its sole application action is `cole.revealChecklistNode`; checklist mutations remain deterministic React controls.
- Use immutable analysis snapshots. Preserve late results as stale history without promoting them when the checklist changed during analysis.
- Use OpenAI only as an optional MVP provider through the Responses API with strict structured output, low reasoning, `store: false`, and deterministic fallback.
- Store the OpenAI key in the OS credential store through `keyring`; store only a credential alias/status in SQLite and never return the saved key to the frontend.
- Remove Obsidian parser, commands, UI, tests, and dependencies from the MVP runtime. Keep Obsidian as a future roadmap connector only.
