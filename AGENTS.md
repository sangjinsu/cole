# AGENTS.md

## Project: Cole

Cole is a local-first AI task assistant.

Cole collects checklist items from Obsidian, Notion, and Dooray, stores normalized tasks in local SQLite, and uses AI to arrange today's work into a simple visual flow.

Cole must not feel like a generic task manager or a full project management dashboard. It should feel like a clean canvas where an AI secretary quietly arranges the user's day.

---

## 0. Product Direction

Cole is a local-first AI task assistant.

Cole collects checklist items from Obsidian, Notion, and Dooray, then asks AI to organize them into a simple visual task flow.

The app does not try to become a full project management tool. It should feel like a clean canvas where AI draws today's work flow.

Cole should answer these questions:

1. What unfinished checklist items exist locally or in connected sources?
2. What should I focus on first?
3. What should come next?
4. What can I finish quickly?
5. Why did Cole choose this order?
6. Can I mark this task done safely?

---

## 1. Architecture Principle

Cole has no central server in MVP.

All data is processed locally on the user's machine. The app may directly call external APIs such as Notion, Dooray, OpenAI, and Claude.

SQLite is the local source of truth for normalized tasks, sync state, local done state, and recommendation cache.

Do not introduce a hosted backend, user account server, or cloud sync layer unless explicitly requested.

Correct mental model:

```txt
User PC
├─ Cole desktop app
│  ├─ React UI
│  ├─ Tauri shell
│  ├─ Rust backend commands
│  ├─ SQLite local database
│  ├─ Obsidian Markdown parser
│  ├─ Notion connector
│  ├─ Dooray connector
│  └─ LLM adapter
│
└─ Direct external API calls
   ├─ Notion API
   ├─ Dooray API
   ├─ OpenAI API
   ├─ Anthropic Claude API
   └─ Optional OpenAI-compatible / LiteLLM endpoint
```

---

## 2. Tech Stack

Use this stack unless the user explicitly changes direction.

- Desktop Framework: Tauri v2
- Frontend: React 18 + TypeScript + Vite
- Package Manager / JS Runtime: Bun
- Styling: Tailwind CSS
- Generative UI: OpenUI for AI-generated VisualCanvas rendering only
- State Management: Zustand
- Server State / API Cache: TanStack Query
- Local Database: SQLite
- SQLite Binding:
  - Preferred: `rusqlite` in the Rust backend for task, source, sync, and recommendation logic.
  - Allowed: Tauri SQL Plugin for simple app-local queries that do not bypass domain rules.
- Backend Runtime: Rust through Tauri commands
- LLM/API Calls:
  - OpenAI API direct call
  - Anthropic Claude API direct call
  - OpenAI-compatible endpoint
  - Future optional LiteLLM endpoint
- Source Connectors:
  - Obsidian local Markdown files
  - Notion API
  - Dooray API
- Testing:
  - Vitest for frontend
  - Rust tests for backend commands, parser logic, database logic, and LLM schema validation
- Code Quality:
  - ESLint
  - Prettier
  - `cargo test`
  - `cargo fmt`
  - `cargo clippy`

Prefer boring, stable libraries. Avoid large frameworks unless they clearly reduce implementation risk.

Use Bun for frontend package management and scripts:

- Use `bun install`, not other package-manager install commands.
- Use `bun run <script>`, not other package-manager run commands.
- Use `bunx` for one-off package execution.

---

## 2.1 OpenUI Direction

Cole should use OpenUI-style generative UI for the upper Visual Canvas.

The AI must not generate arbitrary React code. Instead, Cole should expose a small registered component library that the AI can compose.

Use OpenUI only for the upper `VisualCanvas` area. Do not use OpenUI for core task storage, sync, ranking, source connectors, or source mutation.

The bottom `ChatComposer` remains a normal React component.

The upper `VisualCanvas` may be rendered from OpenUI Lang or an OpenUI-compatible render schema. If OpenUI rendering fails, Cole must fall back to deterministic React rendering of the same validated recommendation flow.

---

## 2.2 OpenUI Component Library

Register only a minimal set of UI components for MVP:

- `TaskFlow`
- `TaskGroup`
- `TaskCard`
- `TaskArrow`
- `SourceBadge`
- `EmptyCanvas`
- `RecommendationNote`

The AI may compose these components to draw today's task flow.

The default visual structure should be:

```txt
Focus -> Next -> Finish
```

---

## 2.3 LLM Rendering Policy

The LLM may generate UI instructions only for the `VisualCanvas`.

Allowed:

- Compose registered OpenUI components
- Group tasks visually
- Explain the recommendation briefly
- Draw simple task flow

Not allowed:

- Generate arbitrary JSX
- Generate executable JavaScript
- Modify tasks directly
- Call external APIs directly
- Write to Obsidian, Notion, or Dooray

Do not pass OpenUI runtime tools or mutation providers to the renderer in MVP.

---

## 3. Project Structure

Use this structure as the initial target.

```txt
cole/
├─ AGENTS.md
├─ README.md
├─ package.json
├─ bun.lock
├─ vite.config.ts
├─ tsconfig.json
├─ index.html
├─ src/
│  ├─ components/
│  │  ├─ AppShell.tsx
│  │  ├─ VisualCanvas.tsx
│  │  ├─ TaskFlowCard.tsx
│  │  ├─ ChatComposer.tsx
│  │  ├─ SourceBadge.tsx
│  │  └─ EmptyState.tsx
│  ├─ lib/
│  │  ├─ openui/
│  │  ├─ api/
│  │  └─ store/
│  ├─ types/
│  └─ main.tsx
├─ src-tauri/
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  └─ src/
│     ├─ commands.rs
│     ├─ db.rs
│     ├─ models.rs
│     ├─ recommendations.rs
│     ├─ sources/
│     │  └─ obsidian.rs
│     └─ main.rs
├─ testdata/
│  ├─ obsidian-vault/
│  ├─ notion-responses/
│  └─ dooray-responses/
├─ docs/
└─ .workspace/
```

Do not introduce a product backend server directory for MVP. Any local runtime used by Tauri must remain embedded in the desktop app.

---

## 4. MVP Scope

Implement only the minimum features required to validate the concept.

MVP features:

1. Load checklist items from one source first.
   - Start with Obsidian Markdown files.
   - Notion and Dooray connectors may be added after the local task model is stable.
2. Normalize checklist items into local SQLite tasks.
3. Show a simple AI-organized task flow:
   - Focus
   - Next
   - Finish
   - Rendered through registered OpenUI components when possible
4. Provide a bottom chat input:
   - "오늘 뭐부터 할까?"
   - "30분 안에 할 일만 보여줘"
   - "Dooray 업무 우선으로 정리해줘"
5. Allow marking a task as done locally.
6. Store task state and AI recommendations in SQLite.

Do not implement complex project management features in MVP.

---

## 5. UI/UX Direction

Cole must use a minimal single-screen interface.

The screen is divided into two main areas:

1. Visual Canvas
   - Takes most of the screen.
   - Looks like a clean white glass board or paper canvas.
   - AI visually draws the user's task flow here using registered OpenUI components.
   - Use only a few task cards.
   - Avoid dense tables, sidebars, dashboards, or complex navigation.
2. Bottom Chat Composer
   - Fixed at the bottom.
   - User interacts with Cole through natural language.
   - Chat input should be simple and calm.
   - It may include a send button and optional source selector.

The app should feel like:

- a clean whiteboard
- a glass sheet
- a paper canvas
- an AI secretary quietly arranging today's work

Cole should communicate like a reliable assistant, not a chatbot.

Good examples:

```txt
Cole has arranged your next steps.
Focus here first.
This task is next because it clears the path for the rest.
This source is read-only, so Cole will only mark it done locally.
```

Avoid overly cute or loud copy.

---

## 6. Frontend Components

Keep the UI component set minimal.

Required components:

- `AppShell`
- `VisualCanvas`
- `ChatComposer`
- `SourceBadge`
- `EmptyState`
- OpenUI component registrations for `TaskFlow`, `TaskGroup`, `TaskCard`, `TaskArrow`, `EmptyCanvas`, and `RecommendationNote`

Avoid implementing these in MVP:

- Full sidebar
- Complex dashboard
- Calendar view
- Kanban board
- Priority matrix
- Multi-tab task management
- Team workspace

---

## 7. Visual Canvas Rules

The `VisualCanvas` is the primary surface of Cole.

Rules:

- Show at most 3 main task groups by default.
- Use simple labels:
  - Focus
  - Next
  - Finish
- Use thin arrows or subtle lines to show task flow.
- Use handwritten-style annotations sparingly.
- Avoid clutter.
- Do not show more than 7 task items on the first screen.
- AI recommendations should feel drawn, not rendered as a spreadsheet.
- Render through OpenUI Lang when valid, and fall back to deterministic React rendering when invalid.

---

## 8. Design System

Use a minimal white glassmorphism style.

Visual rules:

- Background: warm white or very light gray
- Main surface: translucent white glass panel
- Border: thin light gray or pale blue
- Shadow: soft and subtle
- Accent: calm blue
- Text: dark slate
- Corners: large rounded radius
- Motion: subtle fade and slide only

Avoid:

- dark cyberpunk UI
- heavy gradients
- neon effects
- dense dashboards
- too many icons
- strong shadows

Use icons only where they clarify an action. Prefer calm, readable typography over decoration.

---

## 9. Local Task Model

All source items must normalize into one local task model before recommendation.

Minimum fields:

```txt
id
source_id
source_type
external_id
title
body
status
due_at
tags_json
source_location_json
raw_text_hash
sync_state
estimated_minutes
created_at
updated_at
completed_at
```

Task status values:

```txt
todo
done
blocked
archived
```

Source type values:

```txt
obsidian
notion
dooray
manual
```

MVP may keep the model compact, but it must preserve enough source location data to safely identify the original checklist item.

---

## 10. SQLite Requirements

SQLite is the local source of truth.

Required MVP tables:

- `sources`
- `tasks`
- `recommendation_cache`
- `sync_events`

Rules:

- Store raw API tokens only in the OS credential store, never in SQLite.
- Store only credential aliases or secret references in SQLite.
- Use migrations for schema changes.
- Use transactions when syncing or bulk-upserting tasks.
- Keep AI recommendation cache separate from task state.
- Do not let AI output directly update task state without validation.

---

## 11. Source Connectors

Source connectors must normalize external data into local tasks.

MVP order:

1. Obsidian local Markdown files
2. Notion API
3. Dooray API

### Obsidian MVP

Supported checklist syntax:

```md
- [ ] Draft connector notes
- [x] Create SQLite migration
* [ ] Review task flow
```

For each Markdown task, capture:

- raw line
- checked state
- task title
- line number
- heading path when available
- tags when available
- file path
- file hash or line hash

Obsidian write-back is not required for the first MVP. Local done state is enough.

### Notion and Dooray

Notion and Dooray are post-MVP connectors until the local task model and Visual Canvas are stable.

When added, they must follow the same safety rules:

- Pull first.
- Normalize locally.
- Store in SQLite.
- Do not write back automatically.
- Ask for explicit user confirmation before source mutation.

---

## 12. LLM Role

The LLM should only do three things in MVP:

1. Summarize checklist items.
2. Group tasks into Focus, Next, and Finish.
3. Explain briefly why the order was chosen.

The LLM must not directly modify source files or external services.

The LLM output must be validated before rendering.

The LLM must output either the validated AI Recommendation Schema or OpenUI Lang that composes only Cole's registered OpenUI components.

The app must remain useful with LLM access disabled. When LLM access is disabled or unavailable, Cole should still show local tasks using deterministic fallback grouping.

---

## 13. AI Recommendation Schema

Use a simple schema for MVP.

```json
{
  "groups": [
    {
      "id": "focus",
      "title": "Focus",
      "reason": "string",
      "tasks": [
        {
          "taskId": "string",
          "title": "string",
          "estimatedMinutes": 30
        }
      ]
    },
    {
      "id": "next",
      "title": "Next",
      "reason": "string",
      "tasks": []
    },
    {
      "id": "finish",
      "title": "Finish",
      "reason": "string",
      "tasks": []
    }
  ],
  "summary": "string"
}
```

Validation rules:

- `groups` must include only `focus`, `next`, and `finish`.
- Task IDs must refer to tasks that exist in local SQLite.
- Unknown task IDs must be ignored.
- Render validated output only.
- Cache validated recommendations in SQLite.
- When converting to OpenUI Lang, use only the registered Cole OpenUI component library.

---

## 14. Deterministic Fallback

LLM and OpenUI output are assistant layers, not the source of truth.

When AI is disabled, fails, or returns invalid JSON, Cole must still group tasks locally:

- Focus: the highest urgency or most recently changed important task.
- Next: the next few unfinished tasks that appear actionable.
- Finish: short tasks that look easy to complete.

The fallback can be simple in MVP. It must be deterministic and testable.

---

## 15. Chat Composer Behavior

The bottom `ChatComposer` is the primary interaction model.

MVP supported intents:

- Ask what to do first.
- Filter to short tasks.
- Ask for a source-focused arrangement.
- Ask for a calmer or smaller plan.

The chat composer should not become a full chatbot. It guides task arrangement and explains recommendations.

---

## 16. Tauri Command Boundary

Expose use-case level commands to the frontend.

Suggested MVP commands:

```txt
list_tasks
sync_obsidian_source
get_recommendation_flow
create_recommendation_flow
mark_task_done_local
list_sources
create_obsidian_source
update_llm_settings
```

Rules:

- Do not expose raw database methods directly to the frontend.
- Commands should return DTOs, not database rows.
- Commands should validate all frontend input.
- Source mutation commands must require explicit user confirmation.

---

## 17. Security and Privacy

Do not store raw tokens in SQLite.

Store secrets in OS credential storage when available. SQLite may store only references such as:

```txt
secret://openai/default
secret://anthropic/default
secret://notion/personal
secret://dooray/work
```

Never log:

- API keys
- Dooray tokens
- Notion tokens
- OpenAI keys
- Anthropic keys
- full LLM prompts when privacy mode is enabled

Before sending data to an LLM:

- Respect source-level LLM settings.
- Redact secrets.
- Redact emails if privacy mode is enabled.
- Omit source body content unless explicitly allowed.

---

## 18. Testing Requirements

Required tests:

- Obsidian Markdown checklist parser
- SQLite task upsert behavior
- local done state update
- deterministic fallback grouping
- AI recommendation schema validation
- Tauri command input validation
- Visual Canvas rendering of Focus / Next / Finish groups through OpenUI and deterministic fallback
- Chat Composer basic input behavior

Recommended tools:

- Vitest for frontend behavior and rendering
- Rust tests for parser, database, command, and validation logic
- Golden fixtures under `testdata/` for parser behavior

---

## 19. Development Commands

Expected commands:

```bash
# install frontend dependencies
bun install

# run desktop app in development
bun run tauri dev

# run frontend checks
bun run lint
bun run typecheck
bun run test

# run backend checks
cd src-tauri && cargo test
cd src-tauri && cargo fmt --check
cd src-tauri && cargo clippy --all-targets --all-features

# build app
bun run tauri build
```

If scripts are added, keep them thin wrappers around these commands.

---

## 20. Implementation Order

Follow this order unless the user explicitly changes priority.

1. Create Tauri + React + TypeScript project using Bun.
2. Build the single-screen UI:
   - `AppShell`
   - `VisualCanvas`
   - `ChatComposer`
3. Add SQLite local database.
4. Implement local task model.
5. Implement Obsidian Markdown checklist parser.
6. Render tasks as Focus / Next / Finish cards.
7. Add Cole OpenUI component library and VisualCanvas renderer.
8. Add OpenAI-compatible LLM adapter.
9. Add AI recommendation schema validation.
10. Add local done state.
11. Add Notion connector.
12. Add Dooray connector.

Do not start Notion or Dooray before Obsidian, local SQLite tasks, and the Visual Canvas are stable.

---

## 21. Coding Guidelines for Agents

### General

- Keep the architecture local-first.
- Do not introduce a hosted backend.
- Do not hardcode tokens or user paths.
- Keep domain logic independent from UI.
- Keep OpenUI isolated to VisualCanvas rendering.
- Keep provider-specific LLM logic behind a small adapter boundary.
- Keep source-specific parsing behind connector modules.
- Write tests for parser, database, recommendation validation, and local grouping before expanding UI complexity.

### TypeScript

- Use explicit DTO types for Tauri command responses.
- Keep command wrappers in `src/lib/`.
- Keep UI components presentational where possible.
- Use Zustand only for local UI state that genuinely needs sharing.
- Use TanStack Query for command-backed data loading and cache invalidation.

### Rust

- Keep command handlers thin.
- Put database logic under `src-tauri/src/db.rs` until it needs to be split into a folder.
- Put source parsing under `src-tauri/src/sources/`.
- Put LLM request and validation logic under `src-tauri/src/llm/` when the LLM adapter is added.
- Return structured errors suitable for user-facing display.
- Do not log secrets or full private prompts.

### SQLite

- All schema changes must be migrations.
- Do not construct SQL with untrusted string interpolation.
- Use transactions for sync upserts.
- Keep recommendation cache separate from task state.

### LLM

- Validate all structured outputs.
- Cache only validated outputs.
- Respect source-level LLM policy.
- Redact sensitive values before request construction.
- Never allow LLM output to directly mutate source data.
- Never allow LLM output to generate arbitrary JSX or executable JavaScript.

---

## 22. Do Not

- Do not create a central server.
- Do not add login or user account features.
- Do not build a full project management dashboard.
- Do not add complex navigation in MVP.
- Do not show raw task tables as the main UI.
- Do not send source content to LLM without explicit user setting.
- Do not use OpenUI outside the VisualCanvas in MVP.
- Do not allow OpenUI output to call tools or mutations in MVP.
- Do not write back to Dooray, Notion, or Obsidian automatically.
- Do not add calendar, kanban, team, or sync features in MVP.
- Do not make Notion or Dooray required for MVP validation.
- Do not make LLM access required for viewing local tasks.

---

## 23. Definition of Done

A feature is done when:

1. It works in the local desktop app.
2. It does not require a hosted server.
3. It stores required state in SQLite.
4. It has reasonable error handling.
5. It has tests for core logic.
6. It respects LLM privacy policy.
7. It does not leak secrets to logs.
8. It updates docs or `.workspace/` state if behavior changed.

---

## 24. MVP Acceptance Criteria

Cole MVP is acceptable when:

- The app launches as a Tauri desktop app.
- User can register or select an Obsidian vault path.
- Cole parses unchecked and checked Markdown tasks.
- Cole stores normalized tasks in SQLite.
- Cole shows a single-screen Visual Canvas.
- Cole groups tasks into Focus / Next / Finish.
- Cole shows a bottom Chat Composer.
- User can mark a task done locally.
- Cole can use an OpenAI-compatible model to create a validated recommendation flow.
- Cole still works without LLM access using deterministic fallback grouping.
- No central server is required.
- API keys are not stored in SQLite.

---

## 25. Future Extensions

These are allowed after MVP, but must not distort MVP architecture.

- Notion connector
- Dooray connector
- LiteLLM local or remote proxy support
- Calendar integration
- Issue tracker connector
- MCP server mode
- Mobile companion app
- Optional sync server
- Team mode
- End-to-end encrypted cloud sync

If a future sync server is introduced, it must be optional. Cole must continue to work as a local-first app.

---

## 26. Final Architectural Rule

When in doubt, choose:

```txt
Local first
SQLite first
Single screen first
Visual Canvas first
Obsidian first
LLM as assistant, not authority
User confirmation before write-back
No required central server
```

---

## 27. Workspace Management for Agents

Use `.workspace/` as the durable project management area for AI-assisted work.

Required files:

- `.workspace/decisions.md`: record architectural and workflow decisions made by AI agents.
- `.workspace/history.md`: record dated work history, verification results, and major milestones.
- `.workspace/plan.md`: record the current implementation plan and active phase.
- `.workspace/todo.md`: record actionable tasks using Markdown checkbox syntax.

Rules:

1. Read `AGENTS.md` and the `.workspace/` files before starting substantial work.
2. Keep `.workspace/` committed with the repository unless the user explicitly says otherwise.
3. Do not store secrets, API tokens, raw prompts containing private data, or machine-local credentials in `.workspace/`.
4. Update `.workspace/decisions.md` when choosing between meaningful technical alternatives.
5. Update `.workspace/history.md` after completing a meaningful unit of work or verification pass.
6. Update `.workspace/plan.md` when the active project plan changes.
7. Update `.workspace/todo.md` before and after implementation work so another agent can resume safely.
