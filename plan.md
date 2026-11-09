# NEPost — Master Software Development Specification

## 1. Your Role

Act as a Senior Desktop Application Architect, Senior Rust Engineer, Vue.js Engineer, UI/UX Designer, Security Engineer, and QA Engineer.

Your task is to design and develop a professional, production-grade, cross-platform API Development and Testing Desktop Application named **NEPost**.

This is an advanced API development platform inspired by Postman, Insomnia, and Bruno, but it must have its own identity, architecture, and custom UI.

Do not build a simple UI clone or a static prototype. Build a fully functional desktop application with real API execution, persistent storage, request management, testing capabilities, and native desktop integration.

## 2. Product Objectives

- Native desktop application for Windows and Ubuntu Linux.
- Offline-first and local-first architecture.
- Lightweight and resource-efficient.
- Professional custom developer-focused interface.
- Secure credential and token management.
- Extensible modular architecture.
- No mandatory account or cloud connection.
- Support future plugin and synchronization capabilities.

## 3. Mandatory Technology Stack

### Desktop
- Tauri 2
- Rust
- Tokio asynchronous runtime

### Frontend
- Vue 3 Composition API
- TypeScript with strict mode
- Vite
- Tailwind CSS
- shadcn-vue
- Pinia
- Vue Router

### Editors and Visualization
- CodeMirror 6
- JSON syntax highlighting
- Custom JSON tree viewer
- Virtualized lists for large datasets

### Backend
- Reqwest for HTTP requests
- SQLx for SQLite
- Tokio for asynchronous operations
- tokio-tungstenite for WebSocket
- Tonic for gRPC
- QuickJS through a suitable Rust integration for isolated scripting

### Security and Storage
- SQLite for application data
- OS credential manager for secrets
- Secure local file storage
- Database migrations
- Structured logging with sensitive-data redaction

Use stable, mutually compatible dependency versions. Verify current APIs and official documentation before implementing dependencies.

## 4. Architecture Requirements

Implement a modular monolith architecture.

Separate the application into:

1. Presentation layer
2. Frontend state and services
3. Tauri IPC layer
4. Rust application services
5. Protocol execution engines
6. Database repositories
7. Security services
8. Import/export services

Frontend must never directly execute HTTP requests through browser fetch.

All primary HTTP requests must execute through the native Rust backend.

Use typed request and response DTOs.

Implement structured error handling, request IDs, cancellation, progress events, and asynchronous event delivery.

Do not put all business logic inside Tauri command handlers.

Use dependency injection or explicit service-state management where appropriate.

## 5. Complete Feature Requirements

### A. HTTP Request Builder

Implement:
- GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS.
- URL and query parameter editor.
- Headers editor.
- Cookie management.
- Basic authentication.
- Bearer token authentication.
- API key authentication.
- OAuth 2.0.
- JSON, Text, XML, Form Data, URL Encoded, Binary request bodies.
- File upload.
- Custom timeout.
- Redirect configuration.
- Proxy configuration.
- TLS certificate configuration.
- Client certificates.

### B. Response Viewer

Implement:
- Pretty response.
- Raw response.
- Preview mode.
- JSON, XML, HTML and plain-text formatting.
- Image preview.
- Response headers.
- Response cookies.
- Status code.
- Response duration.
- Response size.
- Search within response.
- Copy and save response.
- JSON tree navigation.
- Large response handling.

Do not load enormous responses into a single unbounded UI component.

### C. Workspace and Collections

Implement:
- Multiple workspaces.
- Collections.
- Nested folders.
- Saved requests.
- Request duplication.
- Drag-and-drop sorting.
- Request descriptions.
- Collection-level authentication.
- Request favorites.
- Search.
- Context menus.
- Autosave.

### D. Environment Manager

Implement:
- Global variables.
- Workspace variables.
- Collection variables.
- Folder variables.
- Request variables.
- Multiple environments.
- Environment switching.
- Secure secret variables.
- Variable resolution and precedence.
- Unresolved-variable warnings.

Support syntax such as {{base_url}} and {{token}}.

### E. API Testing Engine

Implement an isolated JavaScript runtime.

Support:
- Pre-request scripts.
- Post-response scripts.
- Assertions.
- Status validation.
- Response JSON extraction.
- Environment manipulation.
- Request chaining.
- Test reports.
- Collection runner.
- Iteration datasets.
- Test execution history.

Provide a Postman-compatible pm-style scripting interface where practical.

Scripts must not receive unrestricted filesystem, process, or network access. Provide controlled APIs and explicit permissions.

Set execution timeouts and memory limits.

### F. Import and Export

Support:
- Postman Collection v2.1.
- OpenAPI 3.x.
- Swagger.
- cURL.
- Insomnia export.
- Native APEX format.
- JSON backup.
- CSV datasets.

Preserve supported data during import/export and clearly report unsupported features.

### G. Advanced Protocols

Implement separate modules for:

- WebSocket.
- Server-Sent Events.
- GraphQL.
- gRPC.

WebSocket must support live connections, messages, message history, and connection status.

GraphQL must support queries, mutations, variables, and headers.

gRPC must support proto import, service discovery, unary calls, and streaming where practical.

### H. Developer Utilities

Implement:
- JWT decoder.
- JSON formatter.
- JSON minifier.
- JSON comparison.
- Base64 encoder/decoder.
- URL encoder/decoder.
- UUID generator.
- Timestamp converter.
- Hash generator.
- Regex tester.
- cURL generator.
- HTTP status reference.

### I. Application Settings

Implement:
- Dark theme.
- Light theme.
- System theme.
- Custom accent colors.
- Editor font settings.
- Keyboard shortcuts.
- Autosave preferences.
- Request defaults.
- Proxy settings.
- Certificate settings.
- History retention.
- Backup and restore.
- Application updates.

## 6. UI/UX Design Direction

Create an entirely custom, modern, premium developer desktop interface.

Do not directly copy Postman or Insomnia.

Visual direction:
- Dark developer-focused interface by default.
- Deep navy/charcoal surfaces.
- Restrained electric-blue accent.
- Clean typography.
- Subtle borders.
- Minimal shadows.
- Compact but comfortable spacing.
- Professional iconography.
- Clear visual hierarchy.

Main layout:

1. Activity bar.
2. Collection sidebar.
3. Request tab workspace.
4. Request editor.
5. Response viewer.
6. Bottom status bar.

Requirements:
- Resizable sidebar.
- Resizable editor and response panels.
- Multiple request tabs.
- Draggable tabs.
- Keyboard-first navigation.
- Context menus.
- Command palette.
- Search.
- Tooltips.
- Loading states.
- Empty states.
- Error states.
- Confirmation dialogs.

Keyboard shortcuts:
- Ctrl+Enter: Send request.
- Ctrl+S: Save request.
- Ctrl+K: Command palette.
- Ctrl+P: Quick search.

The interface must work correctly on smaller desktop windows.

Create reusable components and a consistent design system.

Avoid unnecessary animations and expensive rendering.

## 7. Database Design

Use SQLite with SQLx migrations.

Create appropriate tables for:

- workspaces
- collections
- folders
- requests
- environments
- environment_variables
- request_history
- response_metadata
- cookies
- scripts
- test_runs
- test_results
- grpc_services
- websocket_sessions
- settings

Add appropriate indexes, foreign keys, timestamps, and cascading behavior.

Use transactions for related database changes.

Keep large response bodies in external local storage when appropriate.

Never store authentication secrets in plaintext SQLite columns.

## 8. Project Structure

Use a modular project structure.

Frontend:

src/
  assets/
  components/
    ui/
    editor/
    request/
    response/
    sidebar/
    common/
  layouts/
  views/
  modules/
    request-builder/
    response-viewer/
    collections/
    environments/
    authentication/
    testing/
    websocket/
    graphql/
    grpc/
    importer/
    utilities/
  stores/
  composables/
  services/
  types/
  router/
  utils/
  styles/

Backend:

src-tauri/src/
  commands/
  core/
  modules/
    http/
    websocket/
    graphql/
    grpc/
    scripting/
    authentication/
    collections/
    environments/
    history/
    importer/
    exporter/
  database/
    repositories/
  security/
  utils/

Add tests/, docs/, scripts/, and .github/workflows/.

Organize files by responsibility. Avoid giant files and tightly coupled modules.

## 9. Security Requirements

Security is a mandatory engineering requirement.

- Store credentials using the operating system's secure credential manager.
- Never log passwords, tokens, cookies, or sensitive headers.
- Redact sensitive data from request history and error reports.
- Use safe defaults for TLS verification.
- Require explicit confirmation before disabling certificate validation.
- Restrict filesystem permissions.
- Isolate script execution.
- Validate imported files.
- Prevent path traversal during file operations.
- Use Tauri v2 capabilities with least privilege.
- Avoid unrestricted shell access.
- Do not expose native backend functionality unnecessarily to the frontend.

Because this application intentionally sends requests to user-specified endpoints, support localhost and private development servers. Do not silently block legitimate local development targets.

## 10. Engineering Standards

- TypeScript strict mode.
- Proper Rust error handling.
- No unnecessary any types.
- Reusable components.
- Typed frontend/backend communication.
- Clear naming conventions.
- Database migration versioning.
- Unit tests.
- Integration tests.
- Frontend component tests.
- End-to-end tests.
- Consistent formatting and linting.
- Document important architecture decisions.

Never implement fake functionality.

Do not use hardcoded mock responses as substitutes for actual backend execution.

Do not show a feature as complete unless it works.

Do not leave placeholder buttons that appear functional.

If a feature is not implemented, mark it clearly as planned or disabled.

## 11. Development Execution Strategy

Develop the application incrementally.

### Phase 1: Foundation

Create:
- Tauri project.
- Vue 3 and TypeScript.
- Tailwind and component system.
- Custom main application layout.
- SQLite integration.
- Typed IPC foundation.
- Settings and theme management.

Deliverable: A functioning desktop shell on Windows and Ubuntu.

### Phase 2: HTTP Engine

Create:
- Rust HTTP service.
- Request DTOs.
- Response DTOs.
- Request execution.
- Cancellation.
- Response viewer.
- Request history.

Deliverable: A fully working HTTP API client.

### Phase 3: Workspace

Create:
- Collections.
- Folders.
- Saved requests.
- Environments.
- Variables.
- Authentication.
- Import/export.

Deliverable: Persistent API development workspace.

### Phase 4: Testing

Create:
- Script runtime.
- Pre-request scripts.
- Post-response tests.
- Assertions.
- Collection runner.
- Test reports.

Deliverable: Automated API testing.

### Phase 5: Advanced Protocols

Create:
- WebSocket.
- SSE.
- GraphQL.
- gRPC.
- OpenAPI integration.

Deliverable: Advanced protocol support.

### Phase 6: Production Readiness

Create:
- Security review.
- Performance optimization.
- Backup and restore.
- Native packaging.
- Update infrastructure.
- Documentation.

Deliverable: Production-ready application.

## 12. Important Working Instructions

Work as an engineering team, not as a code generator.

Before implementing:

1. Inspect the existing repository.
2. Understand its current state.
3. Identify missing dependencies.
4. Create an architecture document.
5. Create a development task checklist.
6. Establish coding conventions.
7. Identify technical risks.

Then implement one phase at a time.

For every phase:

1. Explain the implementation plan briefly.
2. Implement the actual code.
3. Run the relevant tests.
4. Fix compilation errors.
5. Verify frontend/backend integration.
6. Update documentation.
7. Mark completed tasks.
8. Identify remaining tasks.

Never overwrite existing working functionality without understanding it.

Do not generate the entire project in one enormous response.

Do not move to the next phase until the current phase is tested and its acceptance criteria are satisfied.

If you encounter an architectural decision, choose the most maintainable and secure approach and document the decision.

If a dependency or API has changed, consult its current official documentation rather than guessing.

## 13. First Task

Start with Phase 1 only.

First inspect the development environment and repository.

Then create:
- Complete architecture.md.
- Development roadmap.
- Initial folder structure.
- Tauri 2 + Vue 3 + TypeScript project.
- Custom dark application shell.
- Sidebar and workspace layout.
- Request tabs UI.
- Request/response panel layout.
- SQLite initialization.
- Initial database migration.
- Typed IPC foundation.

The initial application must launch as a real desktop window.

Test the development build on the current operating system.

Prepare cross-platform build configuration for Windows and Ubuntu.

Do not implement fake HTTP responses yet.

At the end, provide:
- Files created.
- Architecture decisions.
- Commands to run the application.
- Tests performed.
- Current limitations.
- Next development tasks.

**Final instruction:** Prioritize correctness, security, maintainability, performance, and real functionality over generating large amounts of code quickly.