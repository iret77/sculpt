# SCULPT Backlog

## Recently Completed

1. Language Core Semantics (P1)
2. Contract System as First-Class Concept (P1)
3. Convergence Controls in Language (P1)
4. Release Quality Gates (P1)
5. Team-Scale Structure Enablement (P1)
- Multi-file imports via namespace references: `import(namespace.path) [as Alias]`
- Explicit provider namespaces via `use(...)`
- Package/export discovery via `sculpt target packages/exports`
- Legacy magic shorthand removed from default language path
6. Web Stack Foundation (P1)
- Framework-agnostic web standard app IR (`web-app-ir`)
- `web_profile` support (`standard`, `next-app`, `laravel-mvc`)
- Stack adapter discovery via `sculpt target stacks --target web`
7. Pre-LLM Contract Enforcement Hardening (P1)
- `C905`: unknown package namespace in `use(...)`
- `C906`: symbol not exported by package
- Contract-level symbol validation now blocks invalid builds before LLM execution

8. Data-Heavy Benchmark Execution (P1)
- Executed full SCULPT-vs-vibe benchmark run for invoice reconciliation.
- Result: **No-Go** (gate failed 5/5).
- Report: `poc/SCULPT_vs_Vibe_Data_Heavy_Report.md`
- Gate: `poc/gates/data_heavy_vibe_gate.json`


## Validation, Tooling, and Scale

### Execution Plan (Next 2-3 Days)

1. Day 1 - Runtime Data Foundation + Build Safety (P1) [completed]
- Implement deterministic CLI data ops: `csvRead`, `csvHasColumns`, `rowCount`, `sortBy`, `writeCsv`, `writeJson`, `metric`, `processingMs`.
- Add fixed-signature reconciliation op: `reconcileInvoices(...)` as provider-backed operation.
- Enforce compile-time symbol/signature resolution for data/business calls.
- DoD:
  - Unit tests for each data op.
  - Unknown/unbound data ops fail build (no false-positive builds).
  - Reconciliation call path is deterministic and contract-validated.

2. Day 2 - Determinism + Benchmark Harness (P1) [in progress]
- Enforce stable output contract for `reconciliation_report.json` + `exceptions.csv` (schema + ordering).
- Add normalization policy for volatile fields in reproducibility scoring (`generated_at`, `processing_ms`).
- Add benchmark harness command for dataset matrix + reproducibility runs.
- Added CLI command: `sculpt benchmark data-heavy` with dataset matrix + repro runs.
- Added strict artifact validator (required files + report schema + CSV header/order).
- Added normalized hash generation for reproducibility scoring.
- DoD:
  - Normalized reproducibility hash stable for N=5 on accepted runs.
  - Harness emits metrics JSON + gate input JSON in one run.
  - Acceptance validator blocks "success" if mandatory artifacts are missing.

3. Day 3 - ND Guardrails + Competitive Re-Run (P1) [in progress]
- Add strict data-path ND policy (ND disallowed/warned in critical reconciliation logic).
- Improve diagnostics for data workloads (missing symbol/signature/schema errors).
- Re-run full data-heavy benchmark under equal provider/model conditions.
- Progress:
  - Added `@meta nd_critical_path=off|warn|error`.
  - Added semantic ND guardrail diagnostics (`N320`) for deterministic data/business paths.
  - Added warning-vs-error diagnostic handling in compiler semantic phase.
  - Added deterministic runtime signature diagnostics (`C909`) with field-level hints for data ops.
  - Added buildReportJson field diagnostics (`C912`) for clearer schema-level report assembly errors.
  - Added benchmark provider fallback strategy (`openai -> gemini -> stub` when non-strict).
  - Added benchmark metrics markings for fallback usage and provider-attempt traces.
  - Added explicit benchmark failure classification (`infra` vs `product`) and `infra_blocked` gate signaling.
  - Executed full benchmark re-run and gate check with `openai/gpt-4.1-mini` (2026-03-03).
  - Re-run remains blocked by provider quota (`HTTP 429 insufficient_quota`), so matrix/repro acceptance still fail due missing output artifacts.
- DoD:
  - Gate re-run is fully automated and reproducible.
  - Benchmark report refreshed from measured artifacts.
  - Clear pass/fail outcome against pre-registered criteria.

### In Progress

1. Deterministic Data Ops for CLI Target (P1)
- Tracked as Day 1 in "Execution Plan (Next 2-3 Days)".

### Open

2. Persisted Build Telemetry Expansion (P2)
- Surface normalized timestamps and durations in TUI details.
- Add compact per-run trend view (last N builds) for debugging performance drift.
- Progress:
  - Persisted rolling build/run history per script in `dist/<script>/build.history.json` (last 30 entries).
  - TUI details now show normalized timings (`llm/build/run/total`), age, status, and compact trend lines (last 5).

3. Dist Retention Policy (P2)
- Add retention options to `sculpt clean` (age, count, size budget).
- Optional auto-clean behavior configurable in `sculpt.config.json`.
- Progress:
  - Added retention options to CLI clean command:
    - `--max-age-days <n>`
    - `--keep-latest <n>`
    - `--max-size-mb <n>`
  - Retention mode now works without `--all`/input by pruning `dist/` entries safely.
  - Added optional auto-clean policy via `sculpt.config.json` (`clean.auto` + retention settings), applied after successful build/freeze/replay/run.

4. CLI/TUI Regression Coverage (P2)
- Add integration tests for per-script `dist` isolation and run/build behavior parity.
- Add tests for TUI key actions (`Enter`, `B`, `R`, `F`, `P`, `C`) and modal flows.

5. Prompt-Drift Competitive Benchmarking (P2)
- Continuously benchmark SCULPT improvements against prompt-first/vibe baselines.
- Track drift over releases to verify SCULPT remains materially superior in its target category.

6. Baseline Provider Practicality Program (P1)
- Execute `/Users/cwendler/Dev/App/sculpt/docs/SCULPT_Baseline_Provider_Plan.md`.
- Expand contract-exported symbol coverage (`ui/input/data/nd`) for `cli/gui/web`.
- Add provider conformance checks and cross-platform parity gates.
- Progress:
  - Added `data` namespace packages to built-in `cli` and `gui` targets.
  - Extended `web.data` with deterministic batch/data symbols.
  - Added namespaced data-call contract/signature validation and conformance tests.
  - Started B3 with practical UI-kit example set for `cli`, `gui`, `web`.

## Milestone-Priority Queue (Roadmap-Aligned)

1. A. Data-Path Safety Completion (P1)
- ND guardrails for critical data rules (warn/error policy).
- Stronger diagnostics for data/signature/schema failures.
- Strict artifact enforcement defaults in build/run paths.
- Progress:
  - `run` now enforces reconciliation artifact quality when `@meta required_outputs` declares `reconciliation_report.json` + `exceptions.csv`:
    report schema keys, CSV header, and sorted row order are validated (not only file existence).
  - Required-output contract validation now recognizes namespaced writer calls (`data.writeJson` / `data.writeCsv`) as deterministic writers.
  - CLI runtime data ops now resolve namespaced calls in generated output (`data.*`), restoring deterministic artifact generation for the benchmark pipeline.
  - `sculpt benchmark data-heavy --provider stub --target cli` now passes matrix + reproducibility gate (`3/3`, `5/5`, unique hashes `1`, gate `PASS`).
- Exit: data-heavy benchmark passes matrix + reproducibility gate without manual fixes.

2. B. Contract + Namespace Scalability (P1)
- Contract versioning and compatibility checks.
- Scalable namespace/import workflows for large projects.
- Explicit symbol cataloging per target package.
- Progress:
  - Added optional `@meta contract_version=<int>` validation; compile now fails fast when script and active target contract versions diverge.
- Exit: large multi-module projects compile with deterministic symbol resolution and actionable diagnostics.

3. C. Provider Platform Hardening (P1)
- Stabilize LLM/target provider interfaces and fallback policies.
- Normalize provider telemetry in build metadata and TUI.
- Add provider conformance checks.
- Exit: external providers plug in cleanly and pass contract + behavior checks.

4. D. Production-Grade Target Outputs (P1)
- Strengthen `gui` parity across macOS/Windows/Linux.
- Expand `web` stack adapter quality for SSR/CSR profiles.
- Enforce deterministic artifact expectations per target.
- Exit: non-demo app scenarios pass platform-specific quality gates in CI.

5. E. Competitive Benchmark Release Gates (P1)
- Standardize benchmark suite for workflow, data-heavy, and UI scenarios.
- Add release gating based on benchmark thresholds.
- Track SCULPT-vs-vibe deltas by release version.
- Exit: release candidates are blocked automatically when benchmark gates fail.
