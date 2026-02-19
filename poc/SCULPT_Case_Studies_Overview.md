# SCULPT Case Studies Overview

(C) 2026 byte5 GmbH

## Summary Table

| Case Study | Comparison Type | Fairness (Apple-vs-Apple) | SCULPT Pros (Condensed) | SCULPT Cons (Condensed) | Verdict | Details |
|---|---|---|---|---|---|---|
| Incident Triage Assistant (Completed) | SCULPT vs classical TypeScript implementation | Medium | Strong flow modeling, compact intent expression, convergence reporting (`nd_budget`, `confidence`, ND score) | Depends on LLM compile quality/latency, less direct low-level deterministic control | Useful but not universal: good for flow-centric workflows, weaker for implementation-heavy deterministic logic | `poc/POC_Incident_Triage_Report.md` |
| Incident Triage Assistant (Planned strict rerun) | SCULPT vs prompt-first vibe coding | High (target benchmark for SCULPT category) | Expected: better reproducibility, lower drift, better change stability, clearer control during iteration | Risk: if gains are marginal, SCULPT value proposition is not strong enough | Pending (Go/Conditional Go/No-Go gate defined before run) | `poc/SCULPT_vs_Vibe_Incident_Triage.md` + `poc/SCULPT_vs_Vibe_Case_Study_Template.md` |

## Fairness Note

| Comparison | Why It Is / Isn’t Fair |
|---|---|
| SCULPT vs classical language | Only partially fair: different paradigms and optimization goals. Good as capability reference, weak as category benchmark. |
| SCULPT vs vibe coding | Most fair benchmark for SCULPT’s intended category (AI-first, intent-oriented, convergent programming). |

## Current Takeaway
- The first case study validates that SCULPT can produce practical results on real workflow tasks.
- The decisive benchmark is the strict SCULPT-vs-vibe run, because that tests the actual market alternative.
- Continue only if SCULPT demonstrates clear, not marginal, superiority in reproducibility, drift control, and developer workflow quality.

