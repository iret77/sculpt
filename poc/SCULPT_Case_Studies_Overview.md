# SCULPT Case Studies Overview

(C) 2026 byte5 GmbH

## Summary Table

| Case Study | Comparison Type | Fairness (Apple-vs-Apple) | SCULPT Pros (Condensed) | SCULPT Cons (Condensed) | Verdict | Details |
|---|---|---|---|---|---|---|
| Incident Triage Assistant (Completed) | SCULPT vs classical TypeScript implementation | Medium | Strong flow modeling, compact intent expression, convergence reporting (`nd_budget`, `confidence`, ND score) | Depends on LLM compile quality/latency, less direct low-level deterministic control | Useful but not universal: good for flow-centric workflows, weaker for implementation-heavy deterministic logic | [POC Incident Triage Report](POC_Incident_Triage_Report.md) |
| Incident Triage Assistant (Strict rerun completed) | SCULPT vs prompt-first vibe coding | High (target benchmark for SCULPT category) | Clear reproducibility lead (`5/5` stable output hash), fewer corrective iterations, stronger control/readability, much smaller source (`76` vs `204` LOC) | Still depends on LLM compile latency; token-cost comparison not complete yet | **Go**: SCULPT clearly outperformed vibe coding on this task | [SCULPT vs Vibe: Incident Triage](SCULPT_vs_Vibe_Incident_Triage.md) |
| Invoice Reconciliation Batch Processor (Completed) | SCULPT vs prompt-first vibe coding (data-heavy) | Medium (OpenAI quota blocked; SCULPT run used stub provider) | SCULPT source remains compact and intent-structured | Functional acceptance failed (no output artifacts); gate failed 5/5 criteria | **No-Go**: current SCULPT stack is not production-ready for data-heavy reconciliation | [Data-Heavy Report](SCULPT_vs_Vibe_Data_Heavy_Report.md) |

## Fairness Note

| Comparison | Why It Is / Isn’t Fair |
|---|---|
| SCULPT vs classical language | Only partially fair: different paradigms and optimization goals. Good as capability reference, weak as category benchmark. |
| SCULPT vs vibe coding | Most fair benchmark for SCULPT’s intended category (AI-first, intent-oriented, convergent programming). |

## Current Takeaway
- The first case study validates that SCULPT can produce practical results on real workflow tasks.
- The strict SCULPT-vs-vibe benchmark shows a clear SCULPT advantage for this workflow-centric case.
- The benchmark now has a machine-checkable release gate file for reproducibility and strict pass/fail validation.
- Data-heavy benchmark completed with a strict no-go verdict: SCULPT currently lacks required deterministic data-processing execution for this scenario.

## Method Template
Reusable template for future runs:
- [SCULPT vs Vibe Case Study Template](templates/SCULPT_vs_Vibe_Case_Study_Template.md)

## POC Folder Guide
- [POC README](README.md)

## Next Scenario Placeholders
Use these placeholders before running the next benchmark:

| Field | Placeholder |
|---|---|
| Scenario ID | `SCENARIO_<YYYYMMDD>_<slug>` |
| Scenario Name | `<clear task name>` |
| Scenario Category | `<workflow | data-heavy | GUI app | API/backend | mixed>` |
| Business Context | `<one sentence>` |
