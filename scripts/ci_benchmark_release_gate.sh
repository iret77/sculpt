#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SCULPT_BIN="${SCULPT_BIN:-target/debug/sculpt}"
PROVIDER="${SCULPT_PROVIDER:-stub}"
TARGET="${SCULPT_TARGET:-cli}"
SIZES="${SCULPT_BENCH_SIZES:-small,medium,large}"
REPRO_RUNS="${SCULPT_BENCH_REPRO_RUNS:-5}"
METRICS_OUT="${SCULPT_BENCH_METRICS_OUT:-poc/tmp/release_gate_sculpt_metrics.json}"
GATE_IN="${SCULPT_BENCH_GATE_IN:-poc/tmp/release_gate_sculpt_gate_input.json}"
VIBE_METRICS="${SCULPT_VIBE_METRICS:-poc/data_heavy_vibe_metrics.json}"
RESULT_OUT="${SCULPT_RELEASE_GATE_RESULT:-poc/tmp/release_gate_result.json}"

if [[ ! -x "$SCULPT_BIN" ]]; then
  echo "sculpt binary not found at $SCULPT_BIN"
  exit 1
fi

echo "[release-gate] run sculpt benchmark"
"$SCULPT_BIN" benchmark data-heavy \
  --script examples/business/invoice_reconciliation_batch.sculpt \
  --dataset-root poc/data \
  --provider "$PROVIDER" \
  --target "$TARGET" \
  --sizes "$SIZES" \
  --repro-runs "$REPRO_RUNS" \
  --output "$METRICS_OUT" \
  --gate-output "$GATE_IN"

echo "[release-gate] run sculpt internal gate"
"$SCULPT_BIN" gate check "$GATE_IN"

echo "[release-gate] evaluate sculpt-vs-vibe competitive gate"
python3 - <<'PY' "$METRICS_OUT" "$VIBE_METRICS" "$RESULT_OUT"
import json
import sys
from pathlib import Path

sculpt_path = Path(sys.argv[1])
vibe_path = Path(sys.argv[2])
result_path = Path(sys.argv[3])

sculpt = json.loads(sculpt_path.read_text())
vibe = json.loads(vibe_path.read_text())

sculpt_summary = sculpt.get("summary", {})
vibe_runs = vibe.get("runs", [])
vibe_repro = vibe.get("repro", [])

def acceptance_rate(runs):
    if not runs:
        return 0.0
    ok = sum(1 for r in runs if bool(r.get("ok")))
    return ok / len(runs)

def repro_stats(repro_runs):
    ok_runs = [r for r in repro_runs if bool(r.get("ok"))]
    hashes = {str(r.get("hash")) for r in ok_runs if r.get("hash")}
    return {
        "pass": len(ok_runs),
        "unique_hashes": len(hashes),
        "reproducible": len(ok_runs) > 0 and len(hashes) == 1,
    }

vibe_accept = acceptance_rate(vibe_runs)
vibe_repro_stats = repro_stats(vibe_repro)

criteria = [
    {
        "id": "R1",
        "description": "SCULPT acceptance rate is production-grade (>= 0.90)",
        "passed": float(sculpt_summary.get("acceptance_rate", 0.0)) >= 0.90,
        "detail": {
            "sculpt": float(sculpt_summary.get("acceptance_rate", 0.0)),
            "required_min": 0.90,
        },
    },
    {
        "id": "R2",
        "description": "SCULPT reproducibility pass is full (>= 5)",
        "passed": int(sculpt_summary.get("repro_pass", 0)) >= 5,
        "detail": {
            "sculpt": int(sculpt_summary.get("repro_pass", 0)),
            "required_min": 5,
        },
    },
    {
        "id": "R3",
        "description": "SCULPT reproducibility is deterministic (unique hashes <= 1)",
        "passed": int(sculpt_summary.get("repro_unique_hashes", 9999)) <= 1,
        "detail": {
            "sculpt": int(sculpt_summary.get("repro_unique_hashes", 9999)),
            "required_max": 1,
        },
    },
    {
        "id": "R4",
        "description": "SCULPT acceptance is at least vibe acceptance",
        "passed": float(sculpt_summary.get("acceptance_rate", 0.0)) >= vibe_accept,
        "detail": {
            "sculpt": float(sculpt_summary.get("acceptance_rate", 0.0)),
            "vibe": vibe_accept,
        },
    },
    {
        "id": "R5",
        "description": "SCULPT reproducibility uniqueness beats vibe by >= 1 hash",
        "passed": int(sculpt_summary.get("repro_unique_hashes", 9999)) <= max(0, vibe_repro_stats["unique_hashes"] - 1),
        "detail": {
            "sculpt": int(sculpt_summary.get("repro_unique_hashes", 9999)),
            "vibe": int(vibe_repro_stats["unique_hashes"]),
            "required_max": max(0, vibe_repro_stats["unique_hashes"] - 1),
        },
    },
]

failed = [c for c in criteria if not c["passed"]]
result = {
    "name": "data_heavy_release_competitive_gate_v1",
    "sculpt_metrics": str(sculpt_path),
    "vibe_metrics": str(vibe_path),
    "criteria": criteria,
    "pass": len(failed) == 0,
    "failed_count": len(failed),
    "summary": {
        "sculpt_acceptance_rate": float(sculpt_summary.get("acceptance_rate", 0.0)),
        "sculpt_repro_pass": int(sculpt_summary.get("repro_pass", 0)),
        "sculpt_repro_unique_hashes": int(sculpt_summary.get("repro_unique_hashes", 9999)),
        "vibe_acceptance_rate": vibe_accept,
        "vibe_repro_unique_hashes": int(vibe_repro_stats["unique_hashes"]),
    },
}

result_path.parent.mkdir(parents=True, exist_ok=True)
result_path.write_text(json.dumps(result, indent=2) + "\n")

print(json.dumps(result, indent=2))

if failed:
    sys.exit(1)
PY

echo "[release-gate] PASS"
