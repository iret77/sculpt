#!/usr/bin/env python3
import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional


METRIC_KEYS = [
    "sculpt_acceptance_rate",
    "sculpt_repro_pass",
    "sculpt_repro_unique_hashes",
    "vibe_acceptance_rate",
    "vibe_repro_unique_hashes",
]


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def compute_delta(current: dict, previous: Optional[dict]) -> dict:
    current_summary = current.get("summary", {})
    previous_summary = (previous or {}).get("summary", {})
    deltas: dict[str, dict] = {}
    for key in METRIC_KEYS:
        cur = current_summary.get(key)
        prev = previous_summary.get(key) if previous else None
        if isinstance(cur, (int, float)) and isinstance(prev, (int, float)):
            delta = cur - prev
        else:
            delta = None
        deltas[key] = {"current": cur, "previous": prev, "delta": delta}
    return deltas


def render_markdown(report: dict) -> str:
    cur = report["current"]
    prev = report.get("previous")
    lines = [
        "# SCULPT Benchmark Delta Report",
        "",
        "(C) 2026 byte5 GmbH",
        "",
        f"- Generated: `{report['generated_at_utc']}`",
        f"- Current gate file: `{report['current_file']}`",
    ]
    if report.get("previous_file"):
        lines.append(f"- Previous baseline file: `{report['previous_file']}`")
    else:
        lines.append("- Previous baseline file: _none_")
    lines.extend(
        [
            "",
            "## Gate Status",
            "",
            f"- Current pass: `{cur.get('pass')}`",
            f"- Current failed criteria: `{cur.get('failed_count')}`",
        ]
    )
    if prev:
        lines.append(f"- Previous pass: `{prev.get('pass')}`")
        lines.append(f"- Previous failed criteria: `{prev.get('failed_count')}`")
    lines.extend(["", "## Metric Deltas", "", "| Metric | Current | Previous | Delta |", "|---|---:|---:|---:|"])

    for key, vals in report["delta"]["metrics"].items():
        cur_v = vals.get("current")
        prev_v = vals.get("previous")
        delta_v = vals.get("delta")
        cur_s = f"{cur_v:.6f}" if isinstance(cur_v, float) else str(cur_v)
        prev_s = f"{prev_v:.6f}" if isinstance(prev_v, float) else str(prev_v)
        if delta_v is None:
            delta_s = "n/a"
        else:
            delta_s = f"{delta_v:+.6f}" if isinstance(delta_v, float) else f"{delta_v:+d}"
        lines.append(f"| `{key}` | {cur_s} | {prev_s} | {delta_s} |")

    lines.extend(["", "## Criteria Snapshot", ""])
    for criterion in cur.get("criteria", []):
        status = "PASS" if criterion.get("passed") else "FAIL"
        lines.append(f"- `{criterion.get('id')}` {status}: {criterion.get('description')}")
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate SCULPT benchmark delta report.")
    parser.add_argument("--current", required=True, type=Path)
    parser.add_argument("--previous", type=Path)
    parser.add_argument("--out-json", required=True, type=Path)
    parser.add_argument("--out-md", required=True, type=Path)
    args = parser.parse_args()

    current = load_json(args.current)
    previous = load_json(args.previous) if args.previous and args.previous.exists() else None

    report = {
        "name": "sculpt_benchmark_delta_v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "current_file": str(args.current),
        "previous_file": str(args.previous) if previous else None,
        "current": current,
        "previous": previous,
        "delta": {"metrics": compute_delta(current, previous)},
    }

    args.out_json.parent.mkdir(parents=True, exist_ok=True)
    args.out_md.parent.mkdir(parents=True, exist_ok=True)
    args.out_json.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    args.out_md.write_text(render_markdown(report), encoding="utf-8")

    print(json.dumps({
        "name": report["name"],
        "current_pass": current.get("pass"),
        "previous_pass": previous.get("pass") if previous else None,
        "out_json": str(args.out_json),
        "out_md": str(args.out_md),
    }, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
