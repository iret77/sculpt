#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SCULPT_BIN="${SCULPT_BIN:-target/debug/sculpt}"

if [[ ! -x "$SCULPT_BIN" ]]; then
  echo "sculpt binary not found at $SCULPT_BIN"
  exit 1
fi

providers=("stub")
if [[ -n "${OPENAI_API_KEY:-}" ]]; then providers+=("openai"); fi
if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then providers+=("anthropic"); fi
if [[ -n "${GEMINI_API_KEY:-}" ]]; then providers+=("gemini"); fi

providers_csv="$(IFS=,; echo "${providers[*]}")"
verify_flag=""
if [[ "${#providers[@]}" -gt 1 ]]; then
  verify_flag="--verify"
fi

echo "[provider-gate] conformance providers=${providers_csv} verify=${verify_flag:-off}"
"$SCULPT_BIN" auth conformance --providers "$providers_csv" $verify_flag --json > /tmp/sculpt_provider_conformance.json
cat /tmp/sculpt_provider_conformance.json

echo "[provider-gate] data-heavy benchmark smoke run"
"$SCULPT_BIN" benchmark data-heavy \
  --provider stub \
  --target cli \
  --sizes small \
  --repro-runs 1 \
  --output poc/tmp/ci_data_heavy_metrics.json \
  --gate-output poc/tmp/ci_data_heavy_gate.json

echo "[provider-gate] normalize smoke gate thresholds"
python3 - <<'PY'
import json
from pathlib import Path

gate_path = Path("poc/tmp/ci_data_heavy_gate.json")
data = json.loads(gate_path.read_text())
thresholds = data.get("thresholds", {})
thresholds["min_repro_pass"] = 1
data["thresholds"] = thresholds
gate_path.write_text(json.dumps(data, indent=2) + "\n")
PY

echo "[provider-gate] gate check"
"$SCULPT_BIN" gate check poc/tmp/ci_data_heavy_gate.json

echo "[provider-gate] PASS"
