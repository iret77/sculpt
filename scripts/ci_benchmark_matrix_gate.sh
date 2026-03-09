#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SCULPT_BIN="${SCULPT_BIN:-target/debug/sculpt}"
RELEASE_GATE_RESULT="${SCULPT_RELEASE_GATE_RESULT:-poc/tmp/release_gate_result.json}"
INCIDENT_GATE_FILE="${SCULPT_INCIDENT_GATE_FILE:-poc/gates/incident_triage_vibe_gate.json}"
INCIDENT_RESULT="${SCULPT_INCIDENT_GATE_RESULT:-poc/tmp/incident_triage_gate_result.json}"
UI_QUALITY_RESULT="${SCULPT_UI_QUALITY_RESULT:-poc/tmp/target_practical_quality_result.json}"
MATRIX_RESULT="${SCULPT_MATRIX_GATE_RESULT:-poc/tmp/benchmark_matrix_gate_result.json}"

if [[ ! -x "$SCULPT_BIN" ]]; then
  echo "sculpt binary not found at $SCULPT_BIN"
  exit 1
fi

echo "[matrix-gate] data-heavy competitive gate"
"$ROOT_DIR/scripts/ci_benchmark_release_gate.sh"

echo "[matrix-gate] workflow competitive gate (incident triage)"
python3 "$ROOT_DIR/scripts/eval_vibe_gate.py" "$INCIDENT_GATE_FILE" "$INCIDENT_RESULT"

echo "[matrix-gate] ui target quality gate"
"$ROOT_DIR/scripts/ci_target_practical_gates.sh"
cat > "$UI_QUALITY_RESULT" <<'JSON'
{
  "name": "target_practical_quality_v1",
  "pass": true,
  "failed_count": 0
}
JSON

echo "[matrix-gate] aggregate benchmark matrix"
python3 - <<'PY' "$RELEASE_GATE_RESULT" "$INCIDENT_RESULT" "$UI_QUALITY_RESULT" "$MATRIX_RESULT"
import json
import sys
from pathlib import Path

data_heavy = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
workflow = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
ui_quality = json.loads(Path(sys.argv[3]).read_text(encoding="utf-8"))
out = Path(sys.argv[4])

scenarios = [
    {
        "id": "data-heavy",
        "gate": data_heavy.get("name", "data_heavy"),
        "pass": bool(data_heavy.get("pass")),
        "failed_count": int(data_heavy.get("failed_count", 0)),
        "summary": data_heavy.get("summary", {}),
    },
    {
        "id": "workflow",
        "gate": workflow.get("name", "workflow"),
        "pass": bool(workflow.get("pass")),
        "failed_count": int(workflow.get("failed_count", 0)),
    },
    {
        "id": "ui",
        "gate": ui_quality.get("name", "ui_quality"),
        "pass": bool(ui_quality.get("pass")),
        "failed_count": int(ui_quality.get("failed_count", 0)),
    },
]
overall_pass = all(s["pass"] for s in scenarios)

result = {
    "name": "benchmark_matrix_gate_v1",
    "pass": overall_pass,
    "failed_count": sum(1 for s in scenarios if not s["pass"]),
    "scenarios": scenarios,
}
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, indent=2))
if not overall_pass:
    raise SystemExit(1)
PY

echo "[matrix-gate] PASS"
