# NEPost

An offline-first desktop app for building, sending and testing API requests, for **Ubuntu** and
**Windows**. Built with Tauri 2, Rust and Vue 3. Sister app of ENViewer, ENDownloader and
ENPlayer, with the same look and themes.

**Status: Phase 1 (foundation) is done.** The desktop shell, themes, settings, request tabs,
the request editor and the local database work. Sending requests arrives in Phase 2. See
[docs/ROADMAP.md](docs/ROADMAP.md) for progress and [docs/architecture.md](docs/architecture.md)
for how it is built.

## Development

Requirements: Rust (stable), Node 20+, pnpm 9, and on Ubuntu the WebKitGTK development packages
(`libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libayatana-appindicator3-dev libssl-dev`).

```bash
npx pnpm@9.15.0 install
npx pnpm@9.15.0 tauri dev          # run the app with hot reload
```

Without the WebKit packages on the host, build in Docker (the toolchain image shared with
ENDownloader):

```bash
docker build -t endl-build scripts/docker            # once
scripts/docker/run.sh "pnpm install && pnpm tauri build --debug --no-bundle"   # → target/debug/nepost
```

## Tests

```bash
npx pnpm@9.15.0 typecheck && npx pnpm@9.15.0 test   # TypeScript (strict) and Vitest
cargo test --workspace                               # Rust; also regenerates src/types/bindings
scripts/docker/run.sh "dbus-run-session -- scripts/test/smoke-ui.sh"   # real app, headless → target/smoke
```

## Building installers

Push a tag `vX.Y.Z` (after bumping the version in `package.json`, `Cargo.toml` and
`src-tauri/tauri.conf.json`). The `build` workflow tests on Ubuntu and Windows and publishes
`.deb`, AppImage and Windows installer files to a GitHub release.

## Layout

```
src/                      Vue frontend
  components/ui           design-system primitives (same look as the sister apps)
  components/{common,request,response,sidebar,settings}
  layouts/ views/ router/ stores/ (Pinia) composables/ services/ipc.ts types/ (bindings/ generated)
src-tauri/                Rust backend
  src/commands            IPC handlers (thin)
  src/modules             application services (settings; http, collections… in later phases)
  src/database            pool, migrations runner, repositories
  src/security            redaction (secrets and TLS policy later)
  migrations/             versioned SQL
docs/                     architecture, roadmap
scripts/                  Docker toolchain, headless smoke test
```

MIT License · © 2026 M. Estiaque Ahmed Khan
