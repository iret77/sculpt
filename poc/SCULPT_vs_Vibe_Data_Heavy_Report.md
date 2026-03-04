# SCULPT vs Vibe: Data-Heavy Benchmark Report

(C) 2026 byte5 GmbH

Status: `completed (last re-run: 2026-03-03)`

## Scenario

- ID: `SCENARIO_20260222_data_heavy_invoice_reconciliation`
- Task: `Invoice Reconciliation Batch Processor`
- Category: `data-heavy`
- Target: `cli`

## Scope + Inputs

- SCULPT implementation: `examples/business/invoice_reconciliation_batch.sculpt`
- Vibe implementation: `poc/vibe_invoice_reconciliation.js`
- Datasets used:
  - `poc/data/small`
  - `poc/data/medium`
  - `poc/data/large`
- Full run artifacts:
  - `poc/data_heavy_sculpt_metrics.json`
  - `poc/data_heavy_sculpt_repro_metrics.json`
  - `poc/data_heavy_vibe_metrics.json`
  - `poc/data_heavy_vibe_repro_metrics.json`
  - `poc/gates/data_heavy_vibe_gate.json`

## Execution Deviations (Recorded)

1. OpenAI provider was unavailable for SCULPT benchmark build (`insufficient_quota`, HTTP 429).
2. SCULPT benchmark run therefore used `--provider stub` for execution evidence.
3. This keeps the benchmark runnable, but weakens model-tier comparability.
4. Raw failed OpenAI build attempt captured in `poc/data_heavy_openai_attempt.log`.

## Re-Run Update (2026-03-03)

- Command executed:
  - `sculpt benchmark data-heavy --script examples/business/invoice_reconciliation_batch.sculpt --dataset-root poc/data --sizes small,medium,large --repro-runs 5 --provider openai --model gpt-4.1-mini --target cli --output poc/data_heavy_sculpt_metrics.json --gate-output poc/gates/data_heavy_sculpt_gate_input.json`
- Result:
  - Matrix pass: `0 / 3`
  - Repro pass: `0 / 5`
  - Repro unique hashes: `0`
- Gate check:
  - `sculpt gate check poc/gates/data_heavy_sculpt_gate_input.json`
  - Verdict (current logic): `INFRA BLOCKED` when provider quota/availability blocks all evaluable runs.
- Benchmark fallback behavior is now explicit:
  - non-strict runs automatically attempt `openai -> gemini -> stub`
  - metrics include `provider_strategy`, `provider_used`, `fallback_used`, and `provider_attempts`
- Benchmark failure classification is now explicit:
  - run-level `failure_kind`: `none | infra | product`
  - summary-level `infra_blocked` and `infra_failures`
- Blocking reason remains unchanged:
  - OpenAI requests fail with `HTTP 429` / `insufficient_quota`, so no deterministic output artifacts are produced for benchmark scoring.

## Result Snapshot

| Metric | SCULPT | Vibe | Winner |
|---|---:|---:|---|
| Reproducibility score (N=5) | 0 | 5 | Vibe |
| Regression count (lower is better) | 1 | 0 | Vibe |
| Iterations to accepted output (lower is better) | 8 | 1 | Vibe |
| Developer UX score (0..40) | 14 | 31 | Vibe |
| Acceptance quality (0/1) | 0 | 1 | Vibe |

## Hard Measurements

| Measurement | SCULPT | Vibe |
|---|---:|---:|
| Source size (LOC) | 184 | 204 |
| Build+run time small (s) | 0.278 | 0.034 |
| Build+run time medium (s) | 0.282 | 0.497 |
| Build+run time large (s) | 0.295 | 7.902 |
| Output artifacts produced on all dataset sizes | no | yes |

Notes:
- SCULPT runtime profile is fast in this run because the current CLI target emits a minimal stub behavior.
- SCULPT did not generate `reconciliation_report.json` or `exceptions.csv` on any dataset size.
- Vibe output hash normalization drops only `processing_ms` and `generated_at` (time-variant fields).

## Acceptance Review

### SCULPT
- Build: pass (stub provider)
- Run: pass (process exits cleanly)
- Functional acceptance: **fail**
  - no reconciliation outputs generated
  - flow reaches failed state (`errorText`) in step 1

### Vibe
- Build/run: pass
- Functional acceptance: **pass**
  - outputs generated for small/medium/large
  - deterministic report schema and sorted exceptions
  - stress run on large dataset succeeds

## Gate Result

- Command: `sculpt gate check poc/gates/data_heavy_vibe_gate.json`
- Verdict: `FAIL (5 criteria failed)`

Gate file used:
- `poc/gates/data_heavy_vibe_gate.json`

## Decision

- Decision: `no-go`
- Rationale:
  - SCULPT implementation did not meet functional acceptance criteria for this data-heavy benchmark.
  - Vibe baseline met all acceptance criteria and passed stress size.
  - Gate fails all pre-registered criteria.

## Required Follow-Up Actions

1. Implement deterministic data-processing primitives for CLI target (`csvRead`, `writeJson`, `writeCsv`, `sortBy`, `metric`, reconciliation ops) as real provider-backed operations, not inferred placeholders.
2. Add SCULPT compile-time semantic checks for unresolved business operations to prevent false-positive builds.
3. Add benchmark harness command that validates acceptance outputs (`report + exceptions`) before counting a run as successful.
4. Re-run this benchmark with equal model-tier availability once OpenAI quota/billing is active again.
