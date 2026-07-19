# AGENTS.md

## Project: Cole

Cole is a local-first desktop checklist assistant. It keeps one local checklist in SQLite and optionally uses AI to explain relationships and recommend what to do next.

Cole is not a generic task manager, Markdown editor, or project-management dashboard. The checklist is the user's working surface; analysis is a secondary interpretation of that checklist.

## 1. Product Contract

Cole has two views in one desktop window:

1. `Checklist View` is the default and primary workspace.
2. `Analysis View` shows a virtual recommendation order and supporting reasons.

The bottom `ChatComposer` remains visible in both views. Use a small top segmented control for view switching. Do not add a sidebar, multi-tab workspace, hosted backend, account system, or cloud sync layer in the MVP.

Cole must answer:

1. What tasks and groups are in my local checklist?
2. What can I act on now?
3. What should I do next, and why?
4. Which tasks are related or blocked?
5. Can I update the local checklist safely?

When in doubt, choose:

```txt
Local first
SQLite first
Checklist first
Analysis as a read-only interpretation
LLM as assistant, not authority
No required central server
```

## 2. Architecture and Stack

Cole runs entirely on the user's machine:

```txt
Cole desktop app
├─ React 18 + TypeScript + Vite UI
├─ Tauri v2 shell
├─ Rust use-case commands
├─ SQLite local source of truth
├─ OS credential store
├─ deterministic analysis fallback
└─ optional direct OpenAI API call
```

Use this stack unless the user explicitly changes it:

- Package manager/runtime: Bun
- Styling: Tailwind CSS
- Shared UI state: Zustand
- Command-backed cache: TanStack Query
- Database: SQLite through Rust `rusqlite`
- AI provider: OpenAI Responses API only
- Generated analysis UI: OpenUI React Lang with a strict registered library
- Frontend tests: Vitest and Testing Library
- Backend tests: Rust tests

Do not introduce a product server directory. External calls must originate from the embedded Tauri backend.

## 3. Primary User Experience

### Checklist View

The app always opens in Checklist View. The MVP contains one fixed default checklist; checklist creation, deletion, selection, and switching are out of scope.

The checklist is a hierarchical tree of arbitrary depth:

- `task`: actionable node with checkbox state and optional estimated minutes
- `group`: non-checkable node that provides structure and context

Implement the checklist with deterministic React components. It must support:

- inline task and group creation
- inline rename
- task check and uncheck
- task estimate editing
- group collapse and expansion
- node selection
- archive with confirmation when descendants are present
- stable scroll, selection, and expanded state across view switches

New nodes append after the last sibling. Reorder, reparent, drag-and-drop, free-form Markdown editing, and archive restore are outside the MVP.

### Checklist Mutation Rules

- Archive is the only delete behavior; retain archived rows in SQLite.
- A group cannot be checked or completed.
- A parent task with incomplete descendants cannot be completed.
- Do not create a child below a completed task.
- Do not uncheck a descendant while one of its task ancestors is completed.
- Archiving a node with descendants must first return `NON_EMPTY_NODE`; retry only after explicit confirmation with cascade enabled.
- Every mutation must include the caller's expected checklist revision and reject stale revisions.
- Duplicate titles are allowed. Trim titles and require 1-500 characters.
- Task estimates are optional and, when present, must be from 1 to 1440 minutes.

### Analysis View

Analysis View is read-only with respect to checklist data. It may show:

- up to three recommended tasks
- Focus, Next, and Finish roles
- parent-child or inferred execution relationships
- blocked context when relevant
- estimated effort
- a short recommendation reason
- snapshot time and stale state

AI ordering is a virtual order only. It must never reorder or mutate checklist nodes. Clicking an analysis task returns to Checklist View, expands its ancestors, scrolls to the node, focuses it, and briefly highlights it.

### View Transitions and State

- `Cmd/Ctrl+1` switches immediately to Checklist View.
- `Cmd/Ctrl+2` switches immediately to Analysis View.
- Pointer-triggered switching may use a 120 ms cross-fade.
- Respect `prefers-reduced-motion` and remove transitions when requested.
- Switching views must not trigger analysis automatically.
- Preserve checklist scroll, expanded nodes, selected node, composer draft, current snapshot, and analysis zoom for the running session.

## 4. UI Structure and Visual Direction

```txt
AppShell
├─ TopBar
│  ├─ AppBrand
│  ├─ ViewSwitcher
│  └─ SettingsButton
├─ MainView
│  ├─ ChecklistView
│  │  ├─ ChecklistHeader
│  │  ├─ ChecklistTree
│  │  └─ ChecklistRow
│  └─ AnalysisView
│     ├─ AnalysisStatus
│     ├─ OpenUIRenderer
│     └─ AnalysisStaleNotice
└─ ChatComposer
```

Use a quiet paper-and-glass visual system:

- paper: `#FFFFFF`
- app canvas: `#F7F8FA`
- text: `#111827`
- border: `#E3E7EF`
- Focus: `#5B61F6`
- Next: `#4B9CF6`
- Finish: `#39B97F`
- warning: `#D97706`

Keep checklist rows on an opaque paper surface. Use glass treatment only for app chrome such as the top bar, segmented control, composer, and popovers. Do not add fake macOS window controls, decorative gradients, dense cards, or dashboard navigation.

Use semantic buttons, visible focus states, keyboard navigation, and accessible tree semantics. Text must not overlap or truncate essential task meaning at supported window sizes.

## 5. Local Data Contract

SQLite is the source of truth. Use `PRAGMA user_version` migrations and transactions for all schema changes and multi-row mutations.

Required MVP tables:

- `checklists`
- `checklist_nodes`
- `tasks`
- `analysis_snapshots`
- `app_settings`

Use one stable default checklist ID. `checklist_nodes` owns hierarchy, kind, title, sibling order, and archive state. A task node has a 1:1 task projection where `tasks.id == checklist_nodes.id`; a group never creates a task row. Update the node and task rows in the same transaction.

For compatibility during migration, existing manual tasks become root task nodes while preserving IDs, completion state, estimates, and timestamps. Existing source and Obsidian tables may remain physically present until a later cleanup migration, but the MVP runtime must not read from them.

Every checklist has a monotonically increasing revision. Increment it exactly once per successful semantic mutation. Build a canonical SHA-256 checklist hash from active nodes using stable fields such as ID, parent ID, order, kind, title, status, and estimate.

## 6. Tauri Command Boundary

Expose use-case commands, not raw database operations:

```txt
get_default_checklist
create_checklist_node
rename_checklist_node
set_task_checked
set_task_estimate
archive_checklist_node
analyze_checklist
get_latest_analysis_snapshot
get_analysis_snapshot
set_openai_api_key
get_openai_credential_status
delete_openai_api_key
test_openai_connection
```

Commands must validate all input and return DTOs with structured error codes. Checklist mutations must use transactions and expected revision checks. Never expose database rows or secret values directly to the frontend.

## 7. Analysis and Snapshot Policy

Every analysis is tied to an immutable checklist snapshot containing:

```ts
type AnalysisSnapshot = {
  id: string;
  checklistId: string;
  checklistRevision: number;
  checklistHash: string;
  taskIds: string[];
  generatedAt: string;
  provider: "openai" | "deterministic";
  result: AnalysisResult;
};
```

- Cache validated results by checklist hash.
- If the checklist changes while analysis is running, preserve the result as stale history but do not make it the latest active snapshot.
- When the current hash differs from the snapshot hash, show a clear stale notice and `Reanalyze` action.
- Do not silently reuse stale analysis.
- Unknown, done, archived, or duplicate task IDs from an AI response must be removed during Rust validation.
- Render at most three valid recommendations.

### Deterministic Fallback

Cole must remain useful without an API key or network access. The deterministic fallback considers only incomplete actionable leaf tasks and selects, without duplicates:

1. Focus: first task in stable preorder
2. Next: second task in stable preorder
3. Finish: remaining task with the shortest known estimate

The fallback must be deterministic and covered by tests.

## 8. OpenAI and Credential Policy

OpenAI is the only AI provider in the MVP. Do not add Anthropic, OpenAI-compatible endpoints, LiteLLM, or model selection yet.

- Call the OpenAI Responses API from Rust.
- Use model `gpt-5.6`, low reasoning, strict Structured Outputs, `store: false`, and a 20-second timeout.
- Treat the LLM output as untrusted input and validate every ID, relation, and field before persistence or rendering.
- The LLM may recommend and explain; it may not mutate tasks or call tools.
- Store the API key in the OS credential store through `keyring` using service `com.sangjinsu.cole` and account `openai/default`.
- Store only the credential alias and non-secret status metadata in SQLite.
- Never return the raw key to the frontend after it is saved.
- Never log keys, authorization headers, complete private prompts, or raw private checklist content.

## 9. OpenUI Scope

Use OpenUI only in Analysis View. Checklist View, ChatComposer, ViewSwitcher, settings, and all mutation controls remain normal React components.

The strict allowlist is:

- `AnalysisCanvas`
- `PriorityTask`
- `TaskRelation`
- `TaskGroup`
- `BlockedTask`
- `RecommendationReason`
- `AnalysisSummary`
- `SourceReference`

The renderer may emit exactly one application action: `cole.revealChecklistNode`. It may not emit create, edit, check, archive, database, network, or source actions.

Do not allow arbitrary JSX, JavaScript, CSS, unknown components, mutation providers, or runtime tools. If parsing or schema validation fails, render the same validated analysis DTO with deterministic React components.

## 10. Chat Composer

The composer is a focused arrangement control, not a general chatbot.

Checklist View examples:

- analyze today's tasks
- show incomplete tasks
- expand a group
- make a smaller plan

Analysis View examples:

- explain the first recommendation
- show tasks under 30 minutes
- show only prerequisites
- reanalyze with a calmer plan

Composer requests may filter or recompute the virtual analysis. They must not mutate the checklist without a dedicated deterministic confirmation flow.

## 11. MVP Scope

The MVP includes:

1. One default local SQLite checklist.
2. Hierarchical task and group inline CRUD.
3. Local completion and archive rules.
4. Checklist and Analysis view switching with state preservation.
5. Immutable analysis snapshots and stale detection.
6. Deterministic Focus / Next / Finish analysis.
7. Optional validated OpenAI analysis.
8. Analysis-only OpenUI rendering with deterministic fallback.
9. Analysis-card reveal navigation back to the checklist.

The MVP excludes:

- Obsidian, Notion, Dooray, calendar, and issue-tracker connectors
- Markdown parsing, editing, or write-back
- multiple checklists
- reorder, reparent, drag-and-drop, and archive restore
- a hosted backend, login, sync server, team workspace, or mobile app
- automatic source mutation

Obsidian is a future roadmap connector only. Do not keep Obsidian parser, command, UI, test, or dependency code in the MVP runtime.

## 12. Implementation Order

1. Migrate SQLite to the default checklist and hierarchical node model.
2. Implement transactional checklist commands and revision conflict handling.
3. Build Checklist View and inline task/group controls.
4. Add view switching, keyboard shortcuts, and state preservation.
5. Implement deterministic analysis snapshots and stale detection.
6. Add the Analysis-only OpenUI library and reveal action.
7. Add OS credential storage and optional OpenAI analysis.
8. Remove Obsidian runtime code and obsolete dependencies.
9. Complete frontend, Rust, build, and visual verification.

## 13. Coding Guidelines

### TypeScript

- Use explicit DTOs for Tauri command responses.
- Keep command wrappers under `src/lib/`.
- Keep mutation ownership in Checklist View and use TanStack Query invalidation after commands.
- Use Zustand only for shared session UI state.
- Keep components presentational when domain rules belong in Rust.

### Rust

- Keep Tauri command handlers thin.
- Put transactions, revisions, migrations, and domain rules in database/service modules.
- Return structured user-displayable errors.
- Inject or fake network and credential boundaries in tests.
- Do not log secrets or private payloads.

### SQLite

- Use bound parameters, never untrusted SQL interpolation.
- Use transactions for tree mutation, projection updates, migration, and snapshot persistence.
- Keep analysis cache separate from checklist state.

## 14. Testing and Verification

Required coverage:

- migrations and rollback
- arbitrary-depth task/group CRUD
- stale revision conflicts
- parent completion and completed-ancestor rules
- cascade archive confirmation
- canonical checklist hash stability
- deterministic fallback ordering
- snapshot stale/concurrency behavior
- OpenAI request and strict response validation with mocks
- credential storage through a fake adapter
- default Checklist View and accessible tree behavior
- inline CRUD, collapse, selection, scroll, and shortcut state
- stale Analysis notice and reveal navigation
- OpenUI allowlist, deterministic render fallback, and absence of mutation actions
- ChatComposer intent behavior

Run before declaring completion:

```bash
bun install
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

For UI work, verify the actual Tauri app at desktop and narrow window sizes. Check focus order, keyboard actions, overflow, text fit, contrast, and reduced motion.

## 15. Definition of Done

A feature is done only when:

1. It works in the local Tauri desktop app without a hosted server.
2. SQLite persists the required state transactionally.
3. Domain inputs and AI outputs are validated.
4. Secrets remain in the OS credential store and out of logs/SQLite.
5. Core logic and user workflows have tests.
6. Required checks pass with fresh evidence.
7. `README.md` and `.workspace/` reflect the implemented behavior.

## 16. Future Roadmap

After the local checklist and analysis contract is stable, future work may add optional Obsidian, Notion, or Dooray connectors, additional models/providers, multiple checklists, archive restore, or encrypted sync. These must remain optional and must not distort the local-first MVP.

## 17. Workspace Management for Agents

Use `.workspace/` as durable project state. Keep exactly these four committed files:

- `.workspace/decisions.md`
- `.workspace/history.md`
- `.workspace/plan.md`
- `.workspace/todo.md`

Rules:

1. Read `AGENTS.md` and all four workspace files before substantial work.
2. Update decisions for meaningful technical choices.
3. Update plan and todo before and after implementation units.
4. Add dated verification evidence to history.
5. Never store secrets, API tokens, credentials, or private raw prompts in `.workspace/`.
