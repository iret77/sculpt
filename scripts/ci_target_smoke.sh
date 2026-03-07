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

declare -a CASES=(
  "cli examples/games/snake_cli_low_nd_showcase.sculpt"
  "gui examples/getting-started/native_window.sculpt"
  "web examples/web/support_ticket_board.sculpt"
)

for entry in "${CASES[@]}"; do
  target="$(awk '{print $1}' <<< "$entry")"
  script_path="$(awk '{print $2}' <<< "$entry")"
  script_name="$(basename "$script_path" .sculpt)"
  artifact="dist/${script_name}/target.ir.json"

  echo "[target-smoke] build target=${target} script=${script_path} provider=${PROVIDER}"
  "$SCULPT_BIN" build "$script_path" --target "$target" --provider "$PROVIDER" >/tmp/sculpt_target_smoke_"${target}".log

  if [[ ! -f "$artifact" ]]; then
    echo "[target-smoke] missing artifact: $artifact"
    cat /tmp/sculpt_target_smoke_"${target}".log
    exit 1
  fi
done

echo "[target-smoke] PASS"
