#!/usr/bin/env python3
"""
Classic implementation of the same PoC task as incident_triage_assistant.sculpt.
Language: Python (typical imperative CLI style).
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from datetime import datetime, timezone


@dataclass(frozen=True)
class IncidentPlaybook:
  key: str
  title: str
  severity: str
  steps: list[str]


PLAYBOOKS: dict[str, IncidentPlaybook] = {
  "1": IncidentPlaybook(
    key="1",
    title="Service Down",
    severity="SEV-1",
    steps=[
      "Declare SEV-1 and open incident channel.",
      "Start status page incident update.",
      "Assign incident commander and comms owner.",
      "Roll back latest deploy if change is recent.",
    ],
  ),
  "2": IncidentPlaybook(
    key="2",
    title="Error Spike",
    severity="SEV-2",
    steps=[
      "Identify top failing endpoint and error class.",
      "Compare release and config delta from baseline.",
      "Enable degraded mode or feature flag fallback.",
      "Page owning team if sustained longer than 5 minutes.",
    ],
  ),
  "3": IncidentPlaybook(
    key="3",
    title="Latency Increase",
    severity="SEV-2",
    steps=[
      "Check DB, cache, and dependency saturation.",
      "Inspect queue backlog and worker health.",
      "Apply temporary rate limit if saturation continues.",
      "Capture flamegraph/profile before restart actions.",
    ],
  ),
}


def render_menu() -> None:
  print("INCIDENT TRIAGE ASSISTANT")
  print("1 = Service down")
  print("2 = Error spike")
  print("3 = Latency increase")
  print("q = Exit")


def build_result(playbook: IncidentPlaybook) -> dict:
  now = datetime.now(timezone.utc).isoformat()
  return {
    "incident_type": playbook.title,
    "severity": playbook.severity,
    "recommended_actions": playbook.steps,
    "generated_at_utc": now,
  }


def interactive_mode() -> int:
  while True:
    print()
    render_menu()
    choice = input("Select incident type: ").strip().lower()
    if choice == "q":
      print("Session closed.")
      return 0
    if choice not in PLAYBOOKS:
      print("Unknown option. Try again.")
      continue
    result = build_result(PLAYBOOKS[choice])
    print()
    print(f"{result['incident_type']} ({result['severity']})")
    for idx, step in enumerate(result["recommended_actions"], start=1):
      print(f"{idx}. {step}")
    print()
    print("Tip: copy this plan into your incident timeline.")
    return 0


def simulate_mode(choice: str) -> int:
  if choice not in PLAYBOOKS:
    print(json.dumps({"error": f"invalid choice '{choice}'"}, indent=2))
    return 2
  result = build_result(PLAYBOOKS[choice])
  print(json.dumps(result, indent=2))
  return 0


def main(argv: list[str]) -> int:
  parser = argparse.ArgumentParser(
    description="Incident triage helper for first-response action plans."
  )
  parser.add_argument(
    "--simulate",
    choices=sorted(PLAYBOOKS.keys()),
    help="Run non-interactive mode and print JSON for one incident type.",
  )
  args = parser.parse_args(argv)

  if args.simulate:
    return simulate_mode(args.simulate)
  return interactive_mode()


if __name__ == "__main__":
  raise SystemExit(main(sys.argv[1:]))
