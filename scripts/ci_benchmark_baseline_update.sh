#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

CURRENT_RESULT="${SCULPT_RELEASE_GATE_RESULT:-poc/tmp/release_gate_result.json}"
BASELINE_FILE="${SCULPT_RELEASE_GATE_BASELINE:-poc/benchmarks/latest_release_gate_result.json}"
MODE="${SCULPT_RELEASE_GATE_BASELINE_MODE:-candidate}" # candidate|inplace
CANDIDATE_FILE="${SCULPT_RELEASE_GATE_BASELINE_CANDIDATE:-poc/tmp/latest_release_gate_result.candidate.json}"

if [[ ! -f "$CURRENT_RESULT" ]]; then
  echo "[baseline-update] missing release gate result: $CURRENT_RESULT"
  exit 1
fi

python3 - <<'PY' "$CURRENT_RESULT"
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))
if data.get("pass") is not True:
    raise SystemExit("release gate did not pass")
if int(data.get("failed_count", 1)) != 0:
    raise SystemExit("failed_count is not zero")
summary = data.get("summary", {})
if float(summary.get("sculpt_acceptance_rate", 0.0)) < 0.90:
    raise SystemExit("sculpt_acceptance_rate below 0.90")
if int(summary.get("sculpt_repro_pass", 0)) < 5:
    raise SystemExit("sculpt_repro_pass below 5")
if int(summary.get("sculpt_repro_unique_hashes", 9999)) > 1:
    raise SystemExit("sculpt_repro_unique_hashes above 1")
for c in data.get("criteria", []):
    if c.get("passed") is not True:
        raise SystemExit(f"criterion failed: {c.get('id')}")
PY

if [[ "$MODE" == "inplace" ]]; then
  mkdir -p "$(dirname "$BASELINE_FILE")"
  cp "$CURRENT_RESULT" "$BASELINE_FILE"
  echo "[baseline-update] baseline updated in-place: $BASELINE_FILE"
elif [[ "$MODE" == "candidate" ]]; then
  mkdir -p "$(dirname "$CANDIDATE_FILE")"
  cp "$CURRENT_RESULT" "$CANDIDATE_FILE"
  echo "[baseline-update] candidate baseline written: $CANDIDATE_FILE"
else
  echo "[baseline-update] invalid mode '$MODE' (expected candidate|inplace)"
  exit 1
fi
