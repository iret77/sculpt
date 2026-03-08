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

echo "[web-adapter-quality] validating adapter registry"
adapters="$("$SCULPT_BIN" target stacks --target web)"
grep -q "builtin.web.standard@1" <<< "$adapters"
grep -q "provider.web.next@1" <<< "$adapters"
grep -q "provider.web.laravel@1" <<< "$adapters"

check_profile_artifacts() {
  local script_path="$1"
  local expected_profile="$2"
  local script_name
  script_name="$(basename "$script_path" .sculpt)"
  local dist_dir="dist/${script_name}"
  local ir_file="${dist_dir}/ir.json"
  local target_ir_file="${dist_dir}/target.ir.json"
  local main_js="${dist_dir}/main.js"
  local index_html="${dist_dir}/index.html"

  echo "[web-adapter-quality] build ${script_path} (profile=${expected_profile})"
  "$SCULPT_BIN" build "$script_path" --target web --provider "$PROVIDER" >/tmp/sculpt_web_adapter_quality_"${script_name}".log

  test -f "$ir_file"
  test -f "$target_ir_file"
  test -f "$main_js"
  test -f "$index_html"

  python3 - <<'PY' "$ir_file" "$target_ir_file" "$expected_profile"
import json, sys
ir_path, target_ir_path, expected_profile = sys.argv[1], sys.argv[2], sys.argv[3]
with open(ir_path, "r", encoding="utf-8") as f:
    ir = json.load(f)
with open(target_ir_path, "r", encoding="utf-8") as f:
    tir = json.load(f)

profile = str((ir.get("meta") or {}).get("web_profile", ""))
if profile != expected_profile:
    raise SystemExit(f"web_profile mismatch in {ir_path}: got '{profile}', expected '{expected_profile}'")

views = tir.get("views", {})
flow = tir.get("flow", {})
if not isinstance(views, dict) or not views:
    raise SystemExit(f"invalid views in {target_ir_path}")
if not isinstance(flow, dict) or not flow.get("start"):
    raise SystemExit(f"invalid flow.start in {target_ir_path}")
PY

  grep -q "const TARGET = " "$main_js"
  grep -q "window.addEventListener('DOMContentLoaded'" "$main_js"
  grep -q "window.addEventListener('keydown'" "$main_js"
  grep -Fq 'dispatch(`key(${key})`)' "$main_js"
  grep -q "<div id=\"app\"></div>" "$index_html"
  grep -q "<script src=\"main.js\"></script>" "$index_html"
}

check_profile_artifacts "examples/web/web_profile_standard.sculpt" "standard"
check_profile_artifacts "examples/web/incident_status_dashboard.sculpt" "next-app"
check_profile_artifacts "examples/web/support_ticket_board.sculpt" "laravel-mvc"

echo "[web-adapter-quality] PASS"
