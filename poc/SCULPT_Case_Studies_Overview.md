# SCULPT Case Studies Overview

(C) 2026 byte5 GmbH

Status: **historical overview, reclassified 2026-07-12**.

Read [SCULPT Evidence Status](SCULPT_Evidence_Status.md) for the canonical
interpretation of all reports and artifacts.

## Summary

| Case study | Original comparison | What remains useful | Current verdict |
|---|---|---|---|
| Incident Triage vs classical TypeScript | DSL workflow vs hand-written native implementation | Source-structure and workflow-modeling observations | Exploratory only; not an equal-paradigm benchmark |
| Incident Triage vs vibe coding | Structured SCULPT path vs prompt-first regeneration | LOC and iteration observations; benchmark-design lessons | Claimed Go and reproducibility win withdrawn as existence evidence |
| Invoice Reconciliation | SCULPT/stub or provider path vs hand-written JavaScript | Datasets, acceptance scenarios, infra/fallback records | Unresolved; older FAIL and later deterministic-codegen PASS are both non-decision-grade |

## Why The Old Verdicts No Longer Decide

- Vibe coding is not the strongest 2026 competitor.
- The Incident Triage reproducibility arms were asymmetric.
- Deterministic compiler patching and templates confounded model contribution.
- The data-heavy folder contains results from materially different provider and
  implementation paths.
- Token, provider-maintenance, review, and full human costs were incomplete.
- No experiment ablated the typed semantic layer from the agent harness.
- No obligation-level evidence or automatic invalidation system existed.

Raw files are preserved so these conclusions remain auditable. Their historical
PASS, FAIL, Go, or No-Go fields describe only the original local run.

## Current Takeaway

The prototype can parse structured intent, validate contracts, generate target
artifacts, and execute deterministic paths. The old POCs do not show that
SCULPT's proposed language and evidence graph outperform a modern spec-driven
coding agent.

The next valid comparison is the frozen three-arm Brownfield experiment defined
in the
[Convergent Programming Concept](../docs/SCULPT_Convergent_Programming_Concept.md):

1. Full SCULPT.
2. The same harness and structured spec without the typed SCULPT graph.
3. A best-practice spec-driven native coding agent.

No official competitive verdict is authorized before the architecture and
provider gates pass.

## Historical Documents

- [POC Incident Triage Report](POC_Incident_Triage_Report.md)
- [SCULPT vs Vibe: Incident Triage](SCULPT_vs_Vibe_Incident_Triage.md)
- [Data-Heavy Setup](SCULPT_vs_Vibe_Data_Heavy_Setup.md)
- [Data-Heavy Report](SCULPT_vs_Vibe_Data_Heavy_Report.md)
- [Data-Heavy Dataset Specification](SCULPT_vs_Vibe_Data_Heavy_Dataset_Spec.md)
- [Data-Heavy Execution Checklist](SCULPT_vs_Vibe_Data_Heavy_Execution_Checklist.md)

These are historical inputs, not current product claims.
