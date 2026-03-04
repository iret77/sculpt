# SCULPT vs Vibe: Data-Heavy Benchmark Setup

(C) 2026 byte5 GmbH

## Purpose

Set up a strict, repeatable benchmark for a **data-heavy** task where SCULPT competes against prompt-first vibe coding under identical conditions.

This document defines the scenario, metrics, scoring, and execution protocol.

## Scenario

- Scenario ID: `SCENARIO_20260222_data_heavy_invoice_reconciliation`
- Category: `data-heavy`
- Name: `Invoice Reconciliation Batch Processor`
- Target: `cli`

### Task Definition

Build a CLI tool that ingests two CSV inputs:
- `invoices.csv` (expected invoices)
- `payments.csv` (actual payments)

Then produce:
- `reconciliation_report.json`
- `exceptions.csv`
- terminal summary (matched, partial, missing, duplicate, suspicious)

### Functional Acceptance Criteria

1. Correctly matches invoice/payment records by rules (invoice id, customer id, amount, date tolerance).
2. Detects duplicates, partial payments, overpayments, and missing payments.
3. Produces deterministic output format with stable field names and ordering.
4. Handles at least `100k` rows per file without crash.
5. Exposes clear CLI usage and exit codes for success/failure.

## Fixed Conditions (Must Be Equal)

- Same model family tier for both approaches.
- Same max attempts budget.
- Same developer, same machine, same datasets.
- Same acceptance criteria and review checklist.
- Same timebox per approach.

## Measurement Protocol

For each approach collect:

- Iterations until accepted output
- Total generation/build time (seconds)
- Regression count introduced during refinement
- Reproducibility score (N reruns, identical hash count)
- Final acceptance quality (0/1)
- Developer UX score (rubric-based)

## Developer UX Rubric (0..40)

- Change precision (0..10)
- Debuggability (0..10)
- Reproducibility confidence (0..10)
- Cognitive load / editability (0..10)

## Output Artifacts

Store in `poc/`:

- `SCULPT_vs_Vibe_Data_Heavy_Report.md` (filled final report)
- `data_heavy_vibe_prompts.md` (vibe prompt history)
- `data_heavy_vibe_metrics.json`
- `data_heavy_vibe_repro_metrics.json`
- `gates/data_heavy_vibe_gate.json`
- optional raw archive: `artifacts/data_heavy_raw_runs.zip`

## Execution Steps

1. Prepare canonical test datasets (`small`, `medium`, `large`).
2. Run SCULPT implementation and record metrics.
3. Run vibe implementation and record metrics.
4. Run reproducibility reruns (both sides).
5. Fill gate JSON with measured values.
6. Run: `sculpt gate check poc/gates/data_heavy_vibe_gate.json`.
7. Record verdict: `go` / `conditional-go` / `no-go` with rationale.

## Gate Philosophy

- SCULPT does not need to win every metric.
- SCULPT must show a **clear aggregate advantage** in control + reproducibility + developer workflow quality for this category.
- If not, record concrete improvement actions before next run.
