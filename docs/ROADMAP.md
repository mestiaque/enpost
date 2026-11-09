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
- ✅ Tests: Rust (15), frontend (13), headless smoke test of the real app
- ✅ Build configuration for Ubuntu (.deb, AppImage) and Windows (NSIS) in GitHub Actions

## Phase 2: HTTP engine ⬜

- ⬜ `modules/http`: reqwest client, request DTO to execution, response DTO
- ⬜ Request IDs, cancellation, progress events, timings (DNS, connect, TLS, first byte)
- ⬜ Timeout, redirects, proxy, TLS verification and custom CA, client certificates
- ⬜ Bodies: JSON, text, XML, form data, URL encoded, binary, file upload
- ⬜ Response viewer: pretty, raw, preview, JSON tree, image preview, headers, cookies, search, copy, save
- ⬜ Large responses streamed to disk with a preview
- ⬜ CodeMirror 6 editors (body, response)
- ⬜ Request history (redacted) with retention

## Phase 3: Workspace ⬜

- ⬜ Workspaces, collections, nested folders, saved requests, favourites, drag-and-drop order
- ⬜ Autosave, Ctrl+S, descriptions, search, context menus
- ⬜ Environments and variables at every scope, precedence, `{{var}}` highlighting, unresolved warnings
- ⬜ Secret variables in the OS credential manager
- ⬜ Auth: Basic, Bearer, API key, OAuth 2.0; collection-level auth
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
- ⬜ In-app updates (same mechanism as ENViewer and ENDownloader), signed release pipeline
- ⬜ User guide with screenshots
