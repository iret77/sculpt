#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def check_criterion(c):
    sculpt = c.get("sculpt")
    vibe = c.get("vibe")
    op = c.get("operator")
    min_delta = c.get("min_delta", 0)

    if op == "sculpt_gt_vibe":
        return float(sculpt) > float(vibe) and (float(sculpt) - float(vibe)) >= float(min_delta)
    if op == "sculpt_lt_vibe":
        return float(sculpt) < float(vibe) and (float(vibe) - float(sculpt)) >= float(min_delta)
    if op == "sculpt_gte_vibe":
        return float(sculpt) >= float(vibe)
    if op == "sculpt_lte_vibe":
        return float(sculpt) <= float(vibe)
    raise ValueError(f"unsupported operator: {op}")


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: eval_vibe_gate.py <gate.json> <result.json>")

    gate_path = Path(sys.argv[1])
    out_path = Path(sys.argv[2])

    gate = json.loads(gate_path.read_text(encoding="utf-8"))
    criteria = gate.get("criteria") or []
    evaluated = []
    for c in criteria:
        passed = check_criterion(c)
        evaluated.append(
            {
                "id": c.get("id"),
                "description": c.get("description"),
                "operator": c.get("operator"),
                "sculpt": c.get("sculpt"),
                "vibe": c.get("vibe"),
                "min_delta": c.get("min_delta"),
                "passed": passed,
            }
        )

    failed = [c for c in evaluated if not c["passed"]]
    result = {
        "name": gate.get("name", gate_path.stem),
        "source_gate": str(gate_path),
        "study": gate.get("study"),
        "criteria": evaluated,
        "pass": len(failed) == 0,
        "failed_count": len(failed),
    }
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    if failed:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
