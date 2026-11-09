# ENPost roadmap

Each phase ends only when its tests pass and its acceptance criteria are met. ✅ = done and
verified, ⬜ = open.

## Phase 1: Foundation ✅

*Acceptance: a real desktop window on Ubuntu (Windows via CI), with theme and settings
saved in SQLite and a typed IPC path tested end to end.*

- ✅ Tauri 2 + Vue 3 + TypeScript (strict) + Vite project, pnpm, Cargo workspace
- ✅ Tailwind design system ported from ENViewer and ENDownloader (tokens, Dark, Light, Glass, System, accent colours)
- ✅ UI kit: Button, IconButton, Tooltip, Kbd, Input, Switch, Select, Segmented, Dialog, Toaster, confirm dialog, ResizeHandle, EmptyState, WindowControls
- ✅ Layout: title bar (drag, maximise), activity bar, collections sidebar (resizable, collapsible), request tabs, request editor and response panel (resizable split), status bar
- ✅ Request tabs: new, close, close others, duplicate, middle-click close, drag to reorder, unsaved dot, Ctrl+T/W/Tab
- ✅ Request editor: method, URL with two-way Params sync, Params and Headers tables, body type and raw body
- ✅ Command palette (Ctrl+K) and open-tab switcher (Ctrl+P)
- ✅ Settings dialog: General, Appearance, Editor, Requests (TLS needs confirmation), Shortcuts, About
- ✅ SQLite: pool, WAL, foreign keys, embedded migrations, full initial schema (15 tables)
- ✅ Settings service: per-key storage, validation, atomic updates, reset
- ✅ Typed IPC: ts-rs bindings, `call()` command map, `{ kind, message }` errors
- ✅ Security: redaction of headers and URLs, least-privilege capabilities, strict CSP
- ✅ Tests: Rust (15), frontend (13), headless smoke test of the real app (Ubuntu and Windows CI green)
- ✅ Build configuration for Ubuntu (.deb, AppImage) and Windows (NSIS) in GitHub Actions

## Phase 2: HTTP engine ✅

*Acceptance: requests built in the UI run in Rust and their responses show in the viewer,
tested against a local server (Rust integration tests) and in the real app (smoke test).*

- ✅ `modules/http`: reqwest client, typed `HttpRequest` → `HttpResponse` DTOs (ts-rs bindings)
- ✅ Request ids, cancellation at any stage, progress events (`http-progress`), timings (waiting and download)
- ✅ Timeout, redirects (follow / limit / off), TLS verification (turning it off needs confirmation), proxy setting
- ✅ Bodies: JSON, text, XML, URL encoded, form data with file upload, binary file
- ✅ Response viewer: status, time, size, Pretty (CodeMirror), Raw, Preview (HTML without scripts, images), JSON tree (virtualised, copy path or value), headers, cookies, search (Ctrl+F), copy, save to a file
- ✅ Large responses stream to disk; the viewer gets the first 2 MB and says so
- ✅ CodeMirror 6 for the request body and the response
- ✅ Request history: redacted (no auth headers, tokens, cookies or bodies), retention setting, reopen in a tab, clear
- ✅ Typed errors in the UI: timeout, connect, TLS, redirect, invalid request, network
- ✅ Tests: Rust 40 (engine against a local axum server), frontend 24, smoke test sends a real request
- ⬜ Moved on: custom CA certificates and client certificates (Phase 3, with the credential store); a separate DNS/connect/TLS breakdown (reqwest reports only time to headers and download)

## Phase 3: Workspace (in progress)

**3a: collections ✅**
- ✅ Workspaces (first run creates "My Workspace"; switch, create, rename, delete; the last one stays)
- ✅ Collections, nested folders, saved requests; the sidebar tree in stored order
- ✅ Drag and drop: reorder collections, folders and requests; move into folders or other collections (a folder takes its contents along and cannot go into itself)
- ✅ Context menus: new request or folder, rename, duplicate, favourite, delete (with confirmation)
- ✅ Favourites section, search by name or URL (keeps the path to each match)
- ✅ Ctrl+S saves, Save as for a new request (choose collection or folder, or create a collection), autosave after edits (setting), tabs follow renames and deletes
- ✅ Tests: Rust 4 service tests (tree, save, move rules, cascade); frontend tree, drop-index and mapping tests; smoke test creates a collection, saves with Ctrl+S and finds it after a restart
- ⬜ Request descriptions editor (stored already, no editor yet)
- ⬜ Reopen the tabs that were open last time

**Polish (done between 3a and 3b)**
- ✅ Settings like the sister apps: General (on startup: reopen last tabs or a blank request, show sidebar, always on top, autosave, confirm before closing unsaved tabs, update check), Privacy section, full About (build, system, Tauri/WebView, paths with Open buttons, copy diagnostics, developer, source and bug links, open-source credits)
- ✅ UI fixes: icon buttons and Escape in dialogs, focus rings inside bordered fields, app-styled selects and checkboxes, translucent Glass work area, narrow-window response bar

**Still open in Phase 3**
- ⬜ Environments and variables at every scope, precedence, `{{var}}` highlighting, unresolved warnings
- ⬜ Secret variables in the OS credential manager
- ⬜ Auth: Basic, Bearer, API key, OAuth 2.0; collection-level auth
- ⬜ Custom CA certificates and client certificates (secrets in the credential store)
- ⬜ Cookies
- ⬜ Import and export: Postman v2.1, OpenAPI 3, Swagger 2, cURL, Insomnia, ENPost format, JSON backup

## Phase 4: Testing ⬜

- ⬜ QuickJS runtime with memory and time limits, no I/O
- ⬜ `pm`-style API: environment, variables, request, response, expect, test
- ⬜ Pre-request and post-response scripts, assertions, request chaining
- ⬜ Collection runner, iteration datasets (CSV, JSON), reports, run history

## Phase 5: Advanced protocols ⬜

- ⬜ WebSocket (live connection, messages, history, status)
- ⬜ Server-Sent Events
- ⬜ GraphQL (queries, mutations, variables, headers, schema)
- ⬜ gRPC (proto import, reflection, unary, streaming)

## Phase 6: Production readiness ⬜

- ⬜ Security review, performance pass
- ⬜ Developer utilities (JWT, JSON tools, Base64, URL, UUID, timestamps, hashes, regex, cURL generator, status reference)
- ⬜ Backup and restore
- ✅ In-app updates (same mechanism as ENViewer and ENDownloader): daily check, Check now, checksum-verified install (done early, with the settings polish)
- ⬜ Signed release pipeline
- ⬜ User guide with screenshots
