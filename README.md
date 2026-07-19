# Cole

Cole is a local-first desktop checklist assistant. It keeps one hierarchical checklist in local SQLite and helps the user decide what to do next without turning into a full project-management dashboard.

## Product Direction

Cole uses two views in one Tauri window:

- **Checklist** is the default workspace for inline task and group management.
- **Analysis** is a read-only interpretation of the current checklist, showing up to three Focus / Next / Finish recommendations and their reasons.

The checklist remains authoritative. Analysis order is virtual, and selecting an analysis card returns to the matching checklist node. No central server, account, or cloud sync service is required.

## MVP Contract

- One default local SQLite checklist
- Arbitrarily nested task and group nodes
- Inline create, rename, check, estimate, collapse, and archive controls
- Revision-checked transactional mutations
- Parent completion and cascade archive safety rules
- Immutable analysis snapshots with checklist hash and stale detection
- Deterministic analysis when AI is disabled or unavailable
- Optional OpenAI Responses API analysis with validated structured output
- API key storage in the OS credential store, never SQLite
- OpenUI only inside Analysis, with `cole.revealChecklistNode` as its sole app action
- Deterministic React fallback when OpenUI output cannot render

Obsidian is not part of the MVP runtime. Obsidian, Notion, and Dooray connectors remain future roadmap items after the local checklist model is stable.

## Stack

- Tauri v2 and Rust
- React 18, TypeScript, and Vite
- Bun
- Tailwind CSS
- Zustand and TanStack Query
- SQLite through `rusqlite`
- OpenUI React Lang for the Analysis surface only
- Optional OpenAI Responses API integration

## Current Status

The local Checklist + Analysis MVP is implemented.

- Cole opens in Checklist and preserves the tree, selection, scroll position, composer draft, snapshot, and analysis zoom while switching views.
- The checklist supports nested task/group creation, rename, completion, estimates, collapse, and confirmed cascade archive through revision-checked SQLite transactions.
- Analysis renders up to three Focus / Next / Finish cards. Selecting a card returns to and focuses the matching checklist item.
- Immutable snapshots expose stale state, while the safe OpenUI library falls back to deterministic React rendering when needed.
- The responsive layout keeps the checklist usable and Analysis zoom controls visible at narrow window sizes.

The OpenAI provider path is optional. It has been verified with fake credential/provider adapters and request/strict-response contract tests, but this release verification did **not** use a real API key or make a live OpenAI request. The deterministic fallback is tested and remains available without network access.

## Verification

- Frontend: Vitest `38/38`, typecheck, lint, and production build passed. Regression coverage includes rejected async mutations and exact composer draft retention.
- Rust: `cargo test` `24/24`, `cargo fmt --check`, and clippy with `-D warnings` passed.
- Packaging: the final `bun run tauri build` rerun after the async mutation fix passed and produced `Cole.app` and `Cole_0.1.0_aarch64.dmg`; the rebuilt app launched as PID `62796`.
- UI QA: desktop `1180x760` and mobile `390x844` checks found no body overflow or console errors. Checklist opened by default, Analysis rendered three cards, card reveal focused the original tree item, and mobile zoom controls remained visible.
- Review: final Architect verification was `APPROVED`.
- Hygiene: `git diff --check` passed.

## Development

```bash
bun install
bun run tauri dev
bun run test
bun run typecheck
bun run lint
cd src-tauri && cargo test
cd src-tauri && cargo fmt --check
cd src-tauri && cargo clippy --all-targets --all-features
bun run build
bun run tauri build
git diff --check
```

See [AGENTS.md](AGENTS.md) for the product contract, architecture rules, UI behavior, security policy, acceptance criteria, and agent workspace workflow.
