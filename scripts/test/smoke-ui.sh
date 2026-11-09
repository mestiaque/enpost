#!/usr/bin/env bash
# Headless smoke test (inside the build container): the real app starts, the window shows,
# typing a URL fills Params, the palette and settings open, a setting survives a restart.
#   scripts/docker/run.sh "dbus-run-session -- scripts/test/smoke-ui.sh"
set -uo pipefail
OUT="${1:-/src/target/smoke}"
rm -rf "$OUT" /tmp/nphome && mkdir -p "$OUT"
export HOME=/tmp/nphome XDG_RUNTIME_DIR=/tmp/nphome/run XDG_DATA_HOME=/tmp/nphome/.local/share XDG_CONFIG_HOME=/tmp/nphome/.config
mkdir -p "$XDG_RUNTIME_DIR" && chmod 700 "$XDG_RUNTIME_DIR"
DB="$XDG_DATA_HOME/com.mestiaque.nepost/nepost.db"
BIN=/src/target/debug/nepost
trap 'pkill -f "$BIN"; pkill -f "Xvfb :96"' EXIT
Xvfb :96 -screen 0 1320x840x24 >/dev/null 2>&1 &
export DISPLAY=:96
sleep 1
q() { python3 -c 'import sqlite3,sys; [print(*r) for r in sqlite3.connect(sys.argv[1]).execute(sys.argv[2])]' "$DB" "$1"; }
start() {
  "$BIN" >>"$OUT/app.log" 2>&1 &
  for _ in $(seq 1 40); do WID=$(xdotool search --onlyvisible --name NEPost 2>/dev/null | head -1); [ -n "$WID" ] && break; sleep 0.5; done
  [ -n "$WID" ] || { echo "window did not appear"; tail -20 "$OUT/app.log"; exit 1; }
  sleep 2
  eval "$(xdotool getwindowgeometry --shell "$WID")"
}
shot() { sleep "${2:-1.5}"; import -window root "$OUT/$1.png"; echo "shot $1"; }   # root: window grabs can miss overlays

start
echo "window: ${WIDTH}x${HEIGHT}"
shot 01-start
# URL bar: method select is ~104px wide, starting after the activity bar + sidebar.
xdotool mousemove $((X + 700)) $((Y + 101)) click 1
xdotool type --delay 8 "https://api.github.com/repos/tauri-apps/tauri/releases?per_page=5&page=2"
shot 02-url-params
xdotool key ctrl+k; shot 03-palette
xdotool type --delay 20 "glass"; xdotool key Return; shot 04-glass
xdotool key ctrl+comma; shot 05-settings
xdotool key Escape; sleep 0.5
echo "db settings: $(q "SELECT key, value FROM settings")"
echo "schema: $(q "SELECT MAX(version) FROM _sqlx_migrations")"
pkill -f "$BIN"; sleep 1.5
start
shot 06-after-restart
echo "--- log ---"; grep -E "ERROR|WARN|started" "$OUT/app.log" | tail -8
