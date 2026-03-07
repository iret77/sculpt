#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SCULPT_BIN="${SCULPT_BIN:-target/debug/sculpt}"
PROVIDER="${SCULPT_PROVIDER:-stub}"

if [[ ! -x "$SCULPT_BIN" ]]; then
  echo "sculpt binary not found at $SCULPT_BIN"
  exit 1
fi

check_ir_shape() {
  local ir_file="$1"
  local require_transitions="${2:-true}"
  python3 - <<'PY' "$ir_file" "$require_transitions"
import json, sys
path = sys.argv[1]
require_transitions = sys.argv[2].lower() == "true"
with open(path, "r", encoding="utf-8") as f:
    data = json.load(f)
views = data.get("views", {})
flow = data.get("flow", {})
start = flow.get("start", "")
trans = flow.get("transitions", {})
if not isinstance(views, dict) or not views:
    raise SystemExit(f"invalid views in {path}")
if not isinstance(start, str) or not start:
    raise SystemExit(f"invalid flow.start in {path}")
if not isinstance(trans, dict):
    raise SystemExit(f"invalid transitions in {path}")
if require_transitions and not trans:
    raise SystemExit(f"empty transitions in {path}")
PY
}

echo "[target-quality] cli behavior gate"
"$SCULPT_BIN" build examples/games/snake_cli_low_nd_showcase.sculpt --target cli --provider "$PROVIDER" >/tmp/sculpt_quality_cli.log
CLI_DIR="dist/snake_cli_low_nd_showcase"
check_ir_shape "${CLI_DIR}/target.ir.json" true
grep -q "process.stdin.setRawMode" "${CLI_DIR}/main.js"
grep -q "setInterval" "${CLI_DIR}/main.js"

echo "[target-quality] gui behavior gate"
"$SCULPT_BIN" build examples/getting-started/native_window.sculpt --target gui --provider "$PROVIDER" >/tmp/sculpt_quality_gui.log
GUI_DIR="dist/native_window/gui"
if [[ -f "${GUI_DIR}/main.py" ]]; then
  grep -q "tk.Tk()" "${GUI_DIR}/main.py"
  grep -q "root.mainloop()" "${GUI_DIR}/main.py"
elif [[ -f "${GUI_DIR}/Sources/main.swift" ]]; then
  grep -q "WindowGroup" "${GUI_DIR}/Sources/main.swift"
  grep -q "ContentView" "${GUI_DIR}/Sources/main.swift"
else
  echo "[target-quality] missing GUI runtime source in ${GUI_DIR}"
  exit 1
fi
check_ir_shape "dist/native_window/target.ir.json" false

echo "[target-quality] web behavior gate"
"$SCULPT_BIN" build examples/web/support_ticket_board.sculpt --target web --provider "$PROVIDER" >/tmp/sculpt_quality_web.log
WEB_DIR="dist/support_ticket_board"
check_ir_shape "${WEB_DIR}/target.ir.json" false
grep -q "window.addEventListener('keydown'" "${WEB_DIR}/main.js"
grep -Fq 'dispatch(`key(${key})`)' "${WEB_DIR}/main.js"
test -f "${WEB_DIR}/index.html"

echo "[target-quality] PASS"
