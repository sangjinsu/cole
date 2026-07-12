# Todo

Actionable project tasks. Update this file before and after substantial work.

## Product Contract

- [x] Confirm Checklist View as the default and Analysis View as the secondary surface.
- [x] Confirm one default SQLite checklist and local-only task/group CRUD.
- [x] Confirm parent completion, completed-ancestor, and cascade archive rules.
- [x] Restrict OpenUI to Analysis and `cole.revealChecklistNode`.
- [x] Confirm optional OpenAI-only analysis, keyring credential storage, `store: false`, and deterministic fallback.
- [x] Move Obsidian out of the MVP runtime and into the future roadmap.
- [x] Update `AGENTS.md`, `README.md`, and the four-file `.workspace` contract.

## Implementation

- [x] Add versioned SQLite migrations for checklist nodes, revisions, snapshots, and settings.
- [x] Migrate existing manual tasks to root task nodes without losing local state.
- [x] Implement transactional task/group CRUD and revision conflict errors.
- [x] Enforce parent completion and completed-ancestor rules.
- [x] Implement confirmed cascade archive and canonical checklist hashing.
- [x] Build Checklist View with arbitrary-depth tree semantics and inline controls.
- [x] Preserve scroll, expansion, selection, composer draft, snapshot, and zoom state.
- [x] Add Checklist / Analysis segmented switching and `Cmd/Ctrl+1` / `Cmd/Ctrl+2`.
- [x] Implement immutable analysis snapshots, latest-pointer concurrency, and stale notices.
- [x] Implement deterministic Focus / Next / Finish analysis.
- [x] Replace the OpenUI library with Analysis-only components and reveal action.
- [x] Add deterministic Analysis rendering fallback.
- [x] Add OS credential storage and optional OpenAI analysis with strict validation.
- [x] Remove Obsidian runtime code, tests, commands, UI, and obsolete dependencies.
- [x] Update frontend and Rust tests for the new contract.
- [x] Perform desktop and narrow-window Tauri visual QA.

## Verification

- [x] `bun run test` (`38/38`)
- [x] `bun run typecheck`
- [x] `bun run lint`
- [x] `cd src-tauri && cargo test` (`24/24`)
- [x] `cd src-tauri && cargo fmt --check`
- [x] `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings`
- [x] `bun run build`
- [x] `bun run tauri build`
- [x] Launch the packaged `Cole.app` and confirm its process.
- [x] Verify desktop `1180x760` and mobile `390x844` layouts with no body overflow or console errors.
- [x] Verify Analysis card reveal focuses the original checklist tree item and mobile zoom controls remain visible.
- [x] Verify rejected async mutations are handled and the exact composer draft is retained.
- [x] Complete final Architect verification (`APPROVED`).
- [x] `git diff --check`

The optional provider boundary is complete and covered by fake credential/provider and request/strict-response contract tests. A real OpenAI API key and live network call were not part of this verification pass.
