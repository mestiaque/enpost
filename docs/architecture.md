# ENPost architecture

ENPost is a desktop app for building, sending and testing API requests. It works offline and
keeps its data on the user's computer. It runs on Ubuntu and Windows, and it is the sister app
of ENViewer, ENDownloader and ENPlayer. It looks and behaves like them: the same frameless
window, the same Dark, Light, Glass and System themes, accent colours and component style.

This document covers how the app is built and why. [ROADMAP.md](ROADMAP.md) tracks progress.

## 1. Overview

```text
┌──────────────────────── Webview (Vue 3) ────────────────────────┐
│ views / layouts / components      presentation                  │
│ stores (Pinia) · composables      state and UI behaviour        │
│ services/ipc.ts                   the only way to reach Rust    │
└───────────────────────────────┬─────────────────────────────────┘
                                │ Tauri IPC (typed commands, events)
┌───────────────────────────────┴──── Rust (src-tauri) ───────────┐
│ commands/        thin IPC handlers: check input, call a service │
│ modules/         application services (settings; http, …)       │
│   ├─ protocol engines  http · websocket · sse · graphql · grpc  │
│   ├─ scripting         isolated JS runtime (QuickJS)            │
│   └─ import/export     Postman · OpenAPI · cURL · Insomnia      │
│ database/        SQLite pool, migrations, repositories (SQL)    │
│ security/        redaction · secrets (OS keyring) · TLS policy  │
│ core/            AppError, AppState                             │
└─────────────────────────────────────────────────────────────────┘
```

Data flows in one direction: **component → store → `services/ipc` → command → service →
repository / engine**. Results and events come back the same way.

## 2. Layers

| Layer | Where | Responsibility | Must not |
|---|---|---|---|
| Presentation | `src/components`, `src/views`, `src/layouts` | Rendering, user input | Call `invoke` or do network I/O |
| State and services | `src/stores`, `src/composables`, `src/services` | UI state, typed backend calls | Hold business rules that the backend enforces |
| IPC | `src-tauri/src/commands` | Map a command to a service call, map errors | Contain SQL or business logic |
| Application services | `src-tauri/src/modules/*` | Business rules, validation, orchestration | Know about the webview |
| Protocol engines | `modules/{http,websocket,…}` (Phase 2+) | Execute requests, cancellation, progress events | Persist anything directly |
| Repositories | `src-tauri/src/database/repositories` | All SQL | Contain business rules |
| Security | `src-tauri/src/security` | Redaction, secrets, TLS policy | Be bypassed by other modules |

## 3. Key decisions

Each decision states what was chosen, why, and what it costs.

**ADR-1: Vue 3 frontend, with the sister apps' design system ported.**
The specification asks for Vue 3, and the user confirmed it. The sister apps are written in
React, so their design tokens are copied verbatim: the CSS variables for the themes, the
Tailwind colour names, and the Glass styles. Their components (window controls, dialogs,
toasts, settings rows) were rewritten in Vue with the same classes.
*Cost:* those components can't be shared as code. A visual change has to be made in both
places.

**ADR-2: reka-ui primitives instead of the shadcn-vue CLI.**
shadcn-vue is built on reka-ui and copies component code into the project. ENPost uses the
same approach: accessible reka-ui primitives (Dialog, Tooltip, Switch, ContextMenu) wrapped in
`components/ui` with the sister apps' styling. It skips the shadcn theme, which would clash
with that styling.

**ADR-3: SQLx with embedded, versioned migrations.**
Migrations in `src-tauri/migrations` are compiled into the binary (`sqlx::migrate!`) and run at
startup. Queries are written with `sqlx::query` (checked at runtime, covered by tests), not
the compile-time `query!` macros, so building never needs a live database. Every connection
turns on foreign keys, WAL and a busy timeout.

**ADR-4: The whole data model in migration 0001.**
All tables from the specification exist from the start, with foreign keys, cascades and
indexes. Later phases add behaviour, not table rewrites, and data written in Phase 2 already
has its final shape.

**ADR-5: Secrets never go into SQLite.**
A secret variable or cookie row stores only `secret_ref`. The value goes to the OS credential
manager (Secret Service on Ubuntu, Credential Manager on Windows), added in Phase 3. A `CHECK`
constraint rejects any row that is marked secret but also has a plaintext value.

**ADR-6: IPC types generated from Rust.**
DTOs derive `ts_rs::TS`, and `cargo test` writes them to `src/types/bindings`. The frontend's
`call()` maps each command name to its argument and result types. A renamed field breaks the
TypeScript build instead of failing at runtime. Errors cross the boundary as
`{ kind, message }`.

**ADR-7: All API traffic runs in Rust.**
The webview's CSP allows only `ipc:` connections. It cannot `fetch` an API even by mistake.
reqwest executes requests (Phase 2), which also gives real TLS control, proxies, client
certificates and accurate timings.

**ADR-8: Settings stored one key per row.**
Settings are stored as `key → JSON`. Adding a setting needs no migration. An invalid or
outdated value falls back to its default on its own; the other settings are unaffected.
`SettingsService::update` validates the whole result and writes it in one transaction, so a
rejected change leaves nothing half-applied.

**ADR-9: Least-privilege Tauri capabilities.**
The main window may only use window controls (minimise, maximise, drag, close, show and
focus) and ENPost's own commands. ENPost includes no shell, filesystem or HTTP plugin for
the webview. File access (import, export, uploads) goes through specific commands that check
paths.

**ADR-10: Show what is unfinished.**
A feature that isn't built yet is shown disabled, or as an empty state with a "Phase N"
badge. The Send button is disabled with a tooltip until Phase 2. No control pretends to work.

## 4. Conventions

* **Rust**: one error type (`AppError`), and `?` everywhere. No `unwrap` outside tests.
  Commands are `async` and return `AppResult<T>`. SQL lives only in repositories. Code is
  formatted with `cargo fmt`, and `clippy` must be clean.
* **TypeScript**: `strict` and `noUncheckedIndexedAccess` are on. No `any`. Imports use the
  `@/` alias. Components use `<script setup lang="ts">` and typed `defineProps` /
  `defineEmits`. IPC types come from `@/types`, never hand-written copies.
* **Names**: Rust uses `snake_case` and the IPC JSON uses `camelCase` (`serde(rename_all)`).
  Command names are `verb_noun` (`update_settings`). Vue components use PascalCase file names.
* **Database**: ids are TEXT UUIDs and timestamps are milliseconds (UTC). JSON columns hold
  typed DTOs. A migration is never edited once released; add a new one instead.
* **Logging**: no passwords, tokens, cookies or sensitive headers. Anything request-related
  goes through `security::redact` before it is logged or stored in history.
* **Tests**: Rust unit tests sit next to the code (`#[cfg(test)]`). Frontend tests are
  `*.test.ts` next to the file, and app-level ones are in `tests/`. `scripts/test/smoke-ui.sh`
  checks the real app headless.

## 5. Storage locations

| What | Ubuntu | Windows |
|---|---|---|
| Database `enpost.db` | `~/.local/share/com.mestiaque.enpost/` | `%APPDATA%\com.mestiaque.enpost\` |
| Logs | `~/.local/share/com.mestiaque.enpost/logs/` | `%LOCALAPPDATA%\com.mestiaque.enpost\logs\` |
| Large response bodies (Phase 2) | `<data>/bodies/` | `<data>\bodies\` |
| Secrets (Phase 3) | Secret Service (GNOME Keyring / KWallet) | Windows Credential Manager |

## 6. Technical risks

| Risk | Impact | Mitigation |
|---|---|---|
| QuickJS bindings in Rust (`rquickjs`) and sandboxing | Scripts escape limits or hang | Phase 4 spike first: memory limit, interrupt handler for timeouts, no I/O APIs exposed; fuzz the `pm` bridge |
| gRPC without compiled protos (dynamic messages) | Hard to support arbitrary `.proto` files | `prost-reflect` and `tonic` with a dynamic codec; server reflection where available; unary first, then streaming |
| Very large responses (100 MB+) | UI freezes, memory spikes | Stream to disk in Rust, send a preview plus metadata, virtualised viewers, never one huge DOM node |
| OS keyring unavailable (headless Linux, no Secret Service) | Secrets can't be saved | Detect at startup, warn clearly, offer an encrypted local store as a fallback only after explicit consent |
| WebKitGTK and WebView2 differences | Layout or keyboard differences between Ubuntu and Windows | CI on both platforms, same smoke test, avoid browser-specific APIs |
| Import formats (Postman, Insomnia, OpenAPI) are large and loosely specified | Silent data loss | Typed importers, a report of unsupported items, round-trip tests against real exported files |
| Dependency churn (Tauri 3 alpha, sqlx 0.9, vue-router 5) | Breaking upgrades | Exact versions pinned, upgrades in separate commits, CI builds both platforms |
