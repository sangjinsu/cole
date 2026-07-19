# History

Dated work history, verification notes, and major milestones.

## 2026-07-08

- Established `AGENTS.md` and the committed four-file `.workspace` as the project operating contract.
- Bootstrapped the Tauri v2, React 18, TypeScript, Vite, Bun, Rust, and SQLite foundation.
- Implemented an initial Obsidian-oriented VisualCanvas prototype with deterministic recommendations and local done state.
- Published the initial repository and recorded the original foundation verification.

## 2026-07-11

- Approved a new MVP direction centered on one local SQLite checklist with hierarchical task/group inline CRUD.
- Defined Checklist View as the default workspace and Analysis View as a secondary, read-only interpretation.
- Defined revision-checked mutations, parent completion protection, completed-ancestor protection, archive-as-delete, and explicit cascade confirmation.
- Restricted OpenUI to Analysis View with `cole.revealChecklistNode` as its only application action.
- Selected optional OpenAI Responses API analysis with validated structured output, `store: false`, OS keyring storage, and deterministic fallback.
- Removed Obsidian from the MVP runtime contract and retained it only as a future connector.
- Updated `AGENTS.md`, `README.md`, and `.workspace` to describe the approved target while implementation remains in progress.

### Pre-implementation Baseline

- `cd src-tauri && cargo test`: PASS.
- `bun run test`: FAIL, 1 of 6 tests. The failing case was `VisualCanvas.test.tsx` / `routes edit and archive actions from manual tasks in the OpenUI render path`; that obsolete mutation contract is moving to Checklist View coverage.
- `git diff --check`: PASS.

These results describe the baseline before the 2026-07-11 implementation work, not completion evidence for the new MVP contract.

## 2026-07-12

- Completed the local Checklist + Analysis MVP, including the versioned SQLite tree model, transactional inline CRUD, completion/archive safety rules, canonical hash, immutable snapshots, and deterministic analysis.
- Shipped Checklist as the default view with preserved session state, accessible tree navigation, segmented switching, and `Cmd/Ctrl+1` / `Cmd/Ctrl+2` shortcuts.
- Shipped Analysis with up to three Focus / Next / Finish cards, stale handling, safe Analysis-only OpenUI rendering, deterministic React fallback, zoom controls, and reveal navigation back to the original focused `treeitem`.
- Added status-only OS credential commands and the optional OpenAI provider boundary. Fake credential/provider adapters and request/strict-response contract tests passed; no real API key or live OpenAI request was used.
- Removed the Obsidian runtime parser, commands, UI, tests, and obsolete dependencies.

### Completion Evidence

- `bun run test`: PASS, `38/38` Vitest tests, including rejected async mutation handling and exact composer draft retention.
- `bun run typecheck`: PASS.
- `bun run lint`: PASS.
- `bun run build`: PASS.
- `cd src-tauri && cargo test`: PASS, `24/24` Rust tests.
- `cd src-tauri && cargo fmt --check`: PASS.
- `cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings`: PASS.
- Final `bun run tauri build` rerun after the async mutation fix: PASS; generated `Cole.app` and `Cole_0.1.0_aarch64.dmg`.
- Launched the rebuilt `Cole.app` and confirmed its process as PID `62796`.
- gstack browser QA at desktop `1180x760` and mobile `390x844`: no body overflow or console errors.
- UX QA: Checklist opened by default, Analysis showed three recommendation cards, card reveal focused the matching checklist `treeitem`, and mobile zoom controls remained visible.
- Final Architect verification: `APPROVED`.
- `git diff --check`: PASS.
