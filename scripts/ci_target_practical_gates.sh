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

check_ir_richness() {
  local ir_file="$1"
  local min_states="$2"
  local min_transitions="$3"
  python3 - <<'PY' "$ir_file" "$min_states" "$min_transitions"
import json
import sys

path = sys.argv[1]
min_states = int(sys.argv[2])
min_transitions = int(sys.argv[3])

with open(path, "r", encoding="utf-8") as f:
    data = json.load(f)

flows = data.get("flows") or []
if not flows:
    raise SystemExit(f"missing flows in {path}")
flow = flows[0]
states = flow.get("states") or []
if len(states) < min_states:
    raise SystemExit(f"state count too low in {path}: {len(states)} < {min_states}")

transitions = 0
for state in states:
    for stmt in state.get("statements") or []:
        if isinstance(stmt, dict) and "On" in stmt:
            transitions += 1
if transitions < min_transitions:
    raise SystemExit(f"transition count too low in {path}: {transitions} < {min_transitions}")
PY
}

echo "[target-practical] cli practical scenario"
"$SCULPT_BIN" build examples/practical/cli_control_center.sculpt --target cli --provider "$PROVIDER" >/tmp/sculpt_practical_cli.log
CLI_DIR="dist/cli_control_center"
check_ir_richness "${CLI_DIR}/ir.json" 6 10
test -f "${CLI_DIR}/target.ir.json"
grep -q "process.stdin.setRawMode" "${CLI_DIR}/main.js"
grep -q "setInterval" "${CLI_DIR}/main.js"
grep -Fq 'dispatch(`key(' "${CLI_DIR}/main.js"

echo "[target-practical] gui practical scenario"
"$SCULPT_BIN" build examples/practical/gui_service_desk.sculpt --target gui --provider "$PROVIDER" >/tmp/sculpt_practical_gui.log
GUI_DIR="dist/gui_service_desk/gui"
check_ir_richness "dist/gui_service_desk/ir.json" 6 8
test -f "dist/gui_service_desk/target.ir.json"
if [[ -f "${GUI_DIR}/main.py" ]]; then
  grep -q "tk.Tk()" "${GUI_DIR}/main.py"
  grep -q "def dispatch(event):" "${GUI_DIR}/main.py"
  grep -q "elif kind == 'input':" "${GUI_DIR}/main.py"
  grep -q "elif kind == 'table':" "${GUI_DIR}/main.py"
  grep -q "root.bind('<Escape>'" "${GUI_DIR}/main.py"
elif [[ -f "${GUI_DIR}/Sources/main.swift" ]]; then
  grep -q "WindowGroup" "${GUI_DIR}/Sources/main.swift"
  grep -q "func dispatch(_ event: String)" "${GUI_DIR}/Sources/main.swift"
  grep -q "case \"input\":" "${GUI_DIR}/Sources/main.swift"
  grep -q "case \"table\":" "${GUI_DIR}/Sources/main.swift"
  grep -q "onExitCommand" "${GUI_DIR}/Sources/main.swift"
else
  echo "[target-practical] missing GUI runtime source in ${GUI_DIR}"
  exit 1
fi

echo "[target-practical] web practical scenario"
"$SCULPT_BIN" build examples/practical/web_ops_portal.sculpt --target web --provider "$PROVIDER" >/tmp/sculpt_practical_web.log
WEB_DIR="dist/web_ops_portal"
check_ir_richness "${WEB_DIR}/ir.json" 7 10
test -f "${WEB_DIR}/target.ir.json"
test -f "${WEB_DIR}/index.html"
grep -q "<div id=\"app\"></div>" "${WEB_DIR}/index.html"
grep -q "window.addEventListener('DOMContentLoaded'" "${WEB_DIR}/main.js"
grep -q "window.addEventListener('keydown'" "${WEB_DIR}/main.js"
grep -q "function dispatch(event)" "${WEB_DIR}/main.js"

echo "[target-practical] PASS"
