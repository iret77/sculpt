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
  "standard examples/web/web_profile_standard.sculpt"
  "next-app examples/web/incident_status_dashboard.sculpt"
  "laravel-mvc examples/web/support_ticket_board.sculpt"
)

for entry in "${CASES[@]}"; do
  profile="$(awk '{print $1}' <<< "$entry")"
  script_path="$(awk '{print $2}' <<< "$entry")"
  script_name="$(basename "$script_path" .sculpt)"
  ir_file="dist/${script_name}/ir.json"
  web_entry="dist/${script_name}/main.js"

  echo "[web-profiles] build profile=${profile} script=${script_path} provider=${PROVIDER}"
  "$SCULPT_BIN" build "$script_path" --target web --provider "$PROVIDER" >/tmp/sculpt_web_profile_"${script_name}".log

  if [[ ! -f "$web_entry" ]]; then
    echo "[web-profiles] missing web output: ${web_entry}"
    cat /tmp/sculpt_web_profile_"${script_name}".log
    exit 1
  fi

  if [[ ! -f "$ir_file" ]]; then
    echo "[web-profiles] missing ir artifact: ${ir_file}"
    cat /tmp/sculpt_web_profile_"${script_name}".log
    exit 1
  fi

  if ! grep -q "\"web_profile\": \"${profile}\"" "$ir_file"; then
    echo "[web-profiles] expected web_profile=${profile} in ${ir_file}"
    cat /tmp/sculpt_web_profile_"${script_name}".log
    exit 1
  fi
done

echo "[web-profiles] PASS"
