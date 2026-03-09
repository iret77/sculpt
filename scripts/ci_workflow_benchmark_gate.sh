#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

SCULPT_BIN="${SCULPT_BIN:-target/debug/sculpt}"
PROVIDER="${SCULPT_PROVIDER:-stub}"
TARGET="${SCULPT_TARGET:-cli}"
SCRIPT_PATH="${SCULPT_WORKFLOW_SCRIPT:-examples/business/incident_triage_assistant.sculpt}"
REPRO_RUNS="${SCULPT_WORKFLOW_REPRO_RUNS:-5}"
METRICS_OUT="${SCULPT_WORKFLOW_METRICS_OUT:-poc/tmp/workflow_sculpt_metrics.json}"
VIBE_METRICS="${SCULPT_WORKFLOW_VIBE_METRICS:-poc/workflow_vibe_metrics.json}"
RESULT_OUT="${SCULPT_WORKFLOW_GATE_RESULT:-poc/tmp/workflow_gate_result.json}"

if [[ ! -x "$SCULPT_BIN" ]]; then
  echo "sculpt binary not found at $SCULPT_BIN"
  exit 1
fi

if [[ "$TARGET" != "cli" ]]; then
  echo "workflow benchmark currently supports only --target cli"
  exit 1
fi

echo "[workflow-gate] build reproducibility runs"
python3 - <<'PY' "$SCULPT_BIN" "$SCRIPT_PATH" "$TARGET" "$PROVIDER" "$REPRO_RUNS" "$METRICS_OUT"
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

sculpt_bin = sys.argv[1]
script_path = sys.argv[2]
target = sys.argv[3]
provider = sys.argv[4]
repro_runs = int(sys.argv[5])
metrics_out = Path(sys.argv[6])

script_name = Path(script_path).stem
ir_path = Path("dist") / script_name / "ir.json"

required_states = {"Intro", "ServiceDown", "ErrorSpike", "Latency", "Exit"}
required_intro_keys = {"1", "2", "3", "esc"}

def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))

def has_min_action_steps(ir):
    flows = ir.get("flows") or []
    if not flows:
        return False
    states = {s.get("name"): s for s in flows[0].get("states", [])}
    for state_name in ("ServiceDown", "ErrorSpike", "Latency"):
        state = states.get(state_name)
        if not state:
            return False
        step_count = 0
        for stmt in state.get("statements", []):
            expr = stmt.get("Expr")
            if not expr:
                continue
            if expr.get("name") != "ui.text":
                continue
            args = expr.get("args") or []
            if not args:
                continue
            val = args[0].get("value", {})
            text = val.get("String")
            if isinstance(text, str) and text.strip().startswith("-"):
                step_count += 1
        if step_count < 4:
            return False
    return True

def has_required_transitions(ir):
    flows = ir.get("flows") or []
    if not flows:
        return False
    flow = flows[0]
    if flow.get("name") != "Main" or flow.get("start") != "Intro":
        return False
    states = {s.get("name"): s for s in flow.get("states", [])}
    if not required_states.issubset(states.keys()):
        return False

    intro = states["Intro"]
    intro_keys = set()
    for stmt in intro.get("statements", []):
        on = stmt.get("On")
        if not on:
            continue
        ev = on.get("event", {})
        if ev.get("name") != "input.key":
            continue
        args = ev.get("args") or []
        if not args:
            continue
        val = args[0].get("value", {})
        if "Number" in val:
            intro_keys.add(str(int(val["Number"])))
        elif "Ident" in val:
            intro_keys.add(str(val["Ident"]))
    if not required_intro_keys.issubset(intro_keys):
        return False

    for state_name in ("ServiceDown", "ErrorSpike", "Latency"):
        state = states[state_name]
        back_ok = False
        for stmt in state.get("statements", []):
            on = stmt.get("On")
            if not on:
                continue
            if on.get("target") != "Intro":
                continue
            ev = on.get("event", {})
            if ev.get("name") != "input.key":
                continue
            args = ev.get("args") or []
            if not args:
                continue
            val = args[0].get("value", {})
            if val.get("Ident") == "enter":
                back_ok = True
                break
        if not back_ok:
            return False
    return True

runs = []
accepted = 0
hashes = set()

for i in range(1, repro_runs + 1):
    started = time.time()
    proc = subprocess.run(
        [sculpt_bin, "build", script_path, "--target", target, "--provider", provider],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    elapsed_ms = int((time.time() - started) * 1000)
    if proc.returncode != 0 or not ir_path.exists():
        runs.append({
            "run": i,
            "ok": False,
            "elapsed_ms": elapsed_ms,
            "hash": None,
            "reason": "build_failed",
        })
        continue

    ir = read_json(ir_path)
    ok = has_required_transitions(ir) and has_min_action_steps(ir)
    normalized = json.dumps(ir, sort_keys=True, separators=(",", ":"))
    h = hashlib.sha256(normalized.encode("utf-8")).hexdigest()
    hashes.add(h)
    if ok:
        accepted += 1
    runs.append({
        "run": i,
        "ok": ok,
        "elapsed_ms": elapsed_ms,
        "hash": h,
    })

summary = {
    "acceptance_rate": (accepted / repro_runs) if repro_runs else 0.0,
    "repro_pass": accepted,
    "repro_unique_hashes": len({r["hash"] for r in runs if r.get("ok") and r.get("hash")}),
}

out = {
    "name": "workflow_sculpt_metrics_v1",
    "script": script_path,
    "target": target,
    "provider": provider,
    "runs": runs,
    "summary": summary,
}
metrics_out.parent.mkdir(parents=True, exist_ok=True)
metrics_out.write_text(json.dumps(out, indent=2) + "\n", encoding="utf-8")
print(json.dumps(out, indent=2))
PY

echo "[workflow-gate] evaluate competitive criteria"
python3 - <<'PY' "$METRICS_OUT" "$VIBE_METRICS" "$RESULT_OUT"
import json
import sys
from pathlib import Path

sculpt = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
vibe = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
out = Path(sys.argv[3])

ss = sculpt.get("summary", {})
vs = (vibe.get("summary") or {})

vibe_acceptance = float(vs.get("acceptance_rate", 0.0))
vibe_unique_hashes = int(vs.get("repro_unique_hashes", 9999))

criteria = [
    {
        "id": "W1",
        "description": "SCULPT workflow acceptance rate is production-grade (>= 0.90)",
        "passed": float(ss.get("acceptance_rate", 0.0)) >= 0.90,
        "detail": {"sculpt": float(ss.get("acceptance_rate", 0.0)), "required_min": 0.90},
    },
    {
        "id": "W2",
        "description": "SCULPT workflow reproducibility pass is full (>= 5)",
        "passed": int(ss.get("repro_pass", 0)) >= 5,
        "detail": {"sculpt": int(ss.get("repro_pass", 0)), "required_min": 5},
    },
    {
        "id": "W3",
        "description": "SCULPT workflow reproducibility is deterministic (unique hashes <= 1)",
        "passed": int(ss.get("repro_unique_hashes", 9999)) <= 1,
        "detail": {"sculpt": int(ss.get("repro_unique_hashes", 9999)), "required_max": 1},
    },
    {
        "id": "W4",
        "description": "SCULPT workflow acceptance is at least vibe acceptance",
        "passed": float(ss.get("acceptance_rate", 0.0)) >= vibe_acceptance,
        "detail": {"sculpt": float(ss.get("acceptance_rate", 0.0)), "vibe": vibe_acceptance},
    },
    {
        "id": "W5",
        "description": "SCULPT workflow reproducibility uniqueness beats vibe by >= 1 hash",
        "passed": int(ss.get("repro_unique_hashes", 9999)) <= max(0, vibe_unique_hashes - 1),
        "detail": {
            "sculpt": int(ss.get("repro_unique_hashes", 9999)),
            "vibe": vibe_unique_hashes,
            "required_max": max(0, vibe_unique_hashes - 1),
        },
    },
]

failed = [c for c in criteria if not c["passed"]]
result = {
    "name": "workflow_release_competitive_gate_v1",
    "sculpt_metrics": str(sys.argv[1]),
    "vibe_metrics": str(sys.argv[2]),
    "criteria": criteria,
    "pass": len(failed) == 0,
    "failed_count": len(failed),
    "summary": {
        "sculpt_acceptance_rate": float(ss.get("acceptance_rate", 0.0)),
        "sculpt_repro_pass": int(ss.get("repro_pass", 0)),
        "sculpt_repro_unique_hashes": int(ss.get("repro_unique_hashes", 9999)),
        "vibe_acceptance_rate": vibe_acceptance,
        "vibe_repro_unique_hashes": vibe_unique_hashes,
    },
}
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result, indent=2))
if failed:
    raise SystemExit(1)
PY

echo "[workflow-gate] PASS"
