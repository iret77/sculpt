#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

CURRENT_RESULT="${SCULPT_RELEASE_GATE_RESULT:-poc/tmp/release_gate_result.json}"
PREVIOUS_RESULT="${SCULPT_RELEASE_GATE_PREVIOUS:-poc/benchmarks/latest_release_gate_result.json}"
DELTA_JSON_OUT="${SCULPT_RELEASE_DELTA_JSON:-poc/tmp/release_delta_report.json}"
DELTA_MD_OUT="${SCULPT_RELEASE_DELTA_MD:-poc/tmp/release_delta_report.md}"

if [[ ! -f "$CURRENT_RESULT" ]]; then
  echo "[benchmark-delta] missing current release gate result: $CURRENT_RESULT"
  exit 1
fi

echo "[benchmark-delta] current=${CURRENT_RESULT}"
if [[ -f "$PREVIOUS_RESULT" ]]; then
  echo "[benchmark-delta] previous=${PREVIOUS_RESULT}"
else
  echo "[benchmark-delta] previous baseline not found, generating first snapshot report"
  PREVIOUS_RESULT=""
fi

python3 scripts/benchmark_delta_report.py \
  --current "$CURRENT_RESULT" \
  ${PREVIOUS_RESULT:+--previous "$PREVIOUS_RESULT"} \
  --out-json "$DELTA_JSON_OUT" \
  --out-md "$DELTA_MD_OUT"

echo "[benchmark-delta] wrote: $DELTA_JSON_OUT"
echo "[benchmark-delta] wrote: $DELTA_MD_OUT"
