# ENPost

An offline-first desktop app for building, sending and testing API requests, for **Ubuntu** and
**Windows**. Built with Tauri 2, Rust and Vue 3. Sister app of ENViewer, ENSave and
ENPlayer, with the same look and themes.

**Status: Phase 2 (HTTP engine) is done.** ENPost sends real HTTP requests from its Rust
engine (all methods, every body type, file uploads, timeout, redirects, proxy, cancel) and
shows the response: pretty, raw, preview, JSON tree, headers, cookies, save. Sent requests are
kept in a redacted history. Collections and environments come in Phase 3. See
[docs/ROADMAP.md](docs/ROADMAP.md) for progress and [docs/architecture.md](docs/architecture.md)
for how it is built.

## Install and uninstall

ENPost is published as GitHub releases. The commands below always fetch the **latest** version, so
they work on any computer with an internet connection. Installed copies then update themselves
(Settings → General → *Check for updates*).

### Ubuntu / Debian (22.04 or newer), .deb (recommended)

Open a terminal (**Ctrl+Alt+T**) and paste:

```bash
sudo apt update && sudo apt install -y curl
URL=$(curl -fsSL https://api.github.com/repos/mestiaque/enpost/releases/latest | grep -o 'https://[^"]*_amd64\.deb' | head -1)
curl -fL -o /tmp/enpost.deb "$URL"
sudo apt install -y /tmp/enpost.deb
```

`apt` also installs what ENPost needs. Start it from the app grid (search “ENPost”) or run `enpost`.
Enter your login password when asked (nothing is shown while you type; that is normal).
Running the same commands again later upgrades to the newest version.

### Any Linux, AppImage (no installation)

```bash
mkdir -p ~/Applications
URL=$(curl -fsSL https://api.github.com/repos/mestiaque/enpost/releases/latest | grep -o 'https://[^"]*_amd64\.AppImage' | head -1)
curl -fL -o ~/Applications/ENPost.AppImage "$URL"
chmod +x ~/Applications/ENPost.AppImage
~/Applications/ENPost.AppImage
```

### Windows 10 / 11, installer

Open **PowerShell** (Start → type “PowerShell”) and paste:

```powershell
$r = Invoke-RestMethod https://api.github.com/repos/mestiaque/enpost/releases/latest
$a = $r.assets | Where-Object name -like '*-setup.exe' | Select-Object -First 1
Invoke-WebRequest $a.browser_download_url -OutFile "$env:TEMP\$($a.name)"
Start-Process "$env:TEMP\$($a.name)" -ArgumentList '/S' -Wait
```

No administrator rights are needed. ENPost appears in the Start menu.

### Uninstall

**Ubuntu / Debian:**

```bash
sudo apt remove -y en-post
```

**AppImage:** delete the file: `rm ~/Applications/ENPost.AppImage`

**Windows (installer):** Settings → Apps → ENPost → Uninstall, or in PowerShell:

```powershell
$u = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*','HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*' -ErrorAction SilentlyContinue |
  Where-Object DisplayName -like 'ENPost*' | Select-Object -First 1
Start-Process $u.UninstallString.Trim('"') -ArgumentList '/S' -Wait
```

**Also remove settings and data** (optional). This deletes your collections, environments, history and settings. Export what you want to keep first.

```bash
rm -rf ~/.local/share/com.mestiaque.enpost ~/.cache/com.mestiaque.enpost ~/.config/com.mestiaque.enpost                     # Linux
```

```powershell
Remove-Item -Recurse -Force "$env:APPDATA\com.mestiaque.enpost","$env:LOCALAPPDATA\com.mestiaque.enpost" -ErrorAction SilentlyContinue   # Windows
```

## Development

Requirements: Rust (stable), Node 20+, pnpm 9, and on Ubuntu the WebKitGTK development packages
(`libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libayatana-appindicator3-dev libssl-dev`).

```bash
npx pnpm@9.15.0 install
npx pnpm@9.15.0 tauri dev          # run the app with hot reload
```

Without the WebKit packages on the host, build in Docker (the toolchain image shared with
ENSave):

```bash
docker build -t endl-build scripts/docker            # once
scripts/docker/run.sh "pnpm install && pnpm tauri build --debug --no-bundle"   # → target/debug/enpost
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
