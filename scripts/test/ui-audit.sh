#!/usr/bin/env bash
# Screenshots of every main screen in one theme, for a visual review (not an automated check).
#   scripts/docker/run.sh "dbus-run-session -- scripts/test/ui-audit.sh dark"
set -uo pipefail
THEME="${1:-dark}"
OUT="/src/target/audit/$THEME"
rm -rf "$OUT" /tmp/auhome && mkdir -p "$OUT"
export HOME=/tmp/auhome XDG_RUNTIME_DIR=/tmp/auhome/run XDG_DATA_HOME=/tmp/auhome/.local/share XDG_CONFIG_HOME=/tmp/auhome/.config
mkdir -p "$XDG_RUNTIME_DIR" && chmod 700 "$XDG_RUNTIME_DIR"
DB="$XDG_DATA_HOME/com.mestiaque.enpost/enpost.db"
BIN=/src/target/debug/enpost
trap 'pkill -f "$BIN"; pkill -f "Xvfb :93"; pkill -f "http.server 8766"' EXIT
mkdir -p /tmp/ausrv && printf '%s' '{"users":[{"id":1,"name":"Ada Lovelace","email":"ada@example.com","active":true,"roles":["admin","dev"],"manager":null},{"id":2,"name":"Linus Torvalds","email":"linus@example.com","active":false,"roles":["dev"],"manager":1}],"total":2}' > /tmp/ausrv/users.json
(cd /tmp/ausrv && python3 -m http.server 8766 >/dev/null 2>&1 &)
Xvfb :93 -screen 0 1320x840x24 >/dev/null 2>&1 &
export DISPLAY=:93
sleep 1
"$BIN" >/dev/null 2>&1 & sleep 5; pkill -f "$BIN"; sleep 1
python3 -c "import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); c.execute(\"INSERT OR REPLACE INTO settings(key,value,updated_at) VALUES ('theme', ?, 0)\", ('\"$THEME\"',)); c.commit()" "$DB"
"$BIN" >/dev/null 2>&1 &
sleep 6
shot() { sleep "${2:-1.2}"; import -window root "$OUT/$1.png"; }
click() { xdotool mousemove "$1" "$2"; sleep 0.3; xdotool click 1; }
shot 01-empty
# A request with params and headers, then send.
click 700 110; xdotool type --delay 5 "http://127.0.0.1:8766/users.json?page=1&limit=20"
xdotool key ctrl+Return; shot 02-response 2
click 437 156; shot 03-headers-tab
click 1203 466; shot 04-response-headers
click 1139 466; click 456 503; shot 05-tree 1.5
click 415 503; shot 06-raw
# Body tab with JSON (POST)
click 520 156; shot 07-body-none
click 387 110; sleep 0.3; xdotool key Down; xdotool key Return; sleep 0.3; click 520 156; sleep 0.3
xdotool mousemove 430 196; sleep 0.2; xdotool click 1; sleep 0.3; xdotool mousemove 700 300; xdotool click 1; xdotool type --delay 5 '{"name": "Ada", "active": true}'; shot 08-body-json
# Palette, settings sections, about
xdotool key Escape; xdotool key ctrl+k; shot 09-palette
xdotool key Escape; sleep 0.4; xdotool key ctrl+comma; shot 10-settings-general
for i in 2 3 4 5 6; do click 300 $((180 + (i - 1) * 34)); shot "1$i-settings-$i"; done
xdotool key Escape; sleep 0.5
# Sidebar context menu on empty collections area, history panel, small window
click 24 140; shot 17-history
click 24 68; shot 18-collections
xdotool key ctrl+s; shot 19-save-as
xdotool key Escape; sleep 0.4
W=$(xdotool search --onlyvisible --name ENPost | head -1); xdotool windowsize "$W" 860 560; shot 20-small 1.5
