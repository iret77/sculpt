#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SCULPT_BIN="${SCULPT_BIN:-target/debug/sculpt}"
PROVIDER="${SCULPT_PROVIDER:-stub}"
SCRIPT_PATH="examples/getting-started/native_window.sculpt"
SCRIPT_NAME="native_window"
GUI_DIR="dist/${SCRIPT_NAME}/gui"

if [[ ! -x "$SCULPT_BIN" ]]; then
  echo "sculpt binary not found at $SCULPT_BIN"
  exit 1
fi

echo "[gui-parity] build target=gui script=${SCRIPT_PATH} provider=${PROVIDER}"
"$SCULPT_BIN" build "$SCRIPT_PATH" --target gui --provider "$PROVIDER" >/tmp/sculpt_gui_parity.log

if [[ "$(uname -s)" == "Darwin" ]]; then
  exe="${GUI_DIR}/.build/release/SculptGui"
  if [[ ! -f "$exe" ]]; then
    echo "[gui-parity] missing macOS gui executable: $exe"
    cat /tmp/sculpt_gui_parity.log
    exit 1
  fi
  echo "[gui-parity] macOS executable found: $exe"
else
  py_file="${GUI_DIR}/main.py"
  if [[ ! -f "$py_file" ]]; then
    echo "[gui-parity] missing cross-platform gui source: $py_file"
    cat /tmp/sculpt_gui_parity.log
    exit 1
  fi

  if command -v python3 >/dev/null 2>&1; then
    python3 -m py_compile "$py_file"
  elif command -v python >/dev/null 2>&1; then
    python -m py_compile "$py_file"
  elif command -v py >/dev/null 2>&1; then
    py -3 -m py_compile "$py_file"
  else
    echo "[gui-parity] no python runtime available to validate generated GUI script"
    exit 1
  fi
  echo "[gui-parity] python GUI source validated: $py_file"
fi

echo "[gui-parity] PASS"
