# Data-Heavy Benchmark Execution Checklist

(C) 2026 byte5 GmbH

Use this checklist to execute the benchmark consistently.

## 0) Pre-Flight

- [ ] Repo clean enough for benchmark run (no unrelated changes affecting results)
- [ ] Same machine and environment for both approaches
- [ ] Same model tier configured for both approaches
- [ ] Timer + logging method prepared
- [ ] Canonical datasets prepared (`small`, `medium`, `large`)

## 1) Dataset Validation

- [ ] `invoices.csv` and `payments.csv` present
- [ ] Headers match dataset spec exactly
- [ ] Record counts match declared size profile
- [ ] Spot-check edge-case ratios are roughly within target

Artifacts:
- [ ] save dataset metadata snapshot (`rows`, hash, generated_at)

## 2) SCULPT Run

- [ ] Implement/prepare SCULPT solution for task
- [ ] Run on `small` dataset until accepted correctness
- [ ] Run benchmark on `medium`
- [ ] Run stress pass on `large`
- [ ] Record iterations to accepted output
- [ ] Record total generation/build/run time
- [ ] Record regressions introduced during refinement
- [ ] Record final acceptance quality (0/1)
- [ ] Record developer UX score (0..40)

Artifacts:
- [ ] SCULPT script(s)
- [ ] output files
- [ ] metrics summary

## 3) Vibe Run

- [ ] Use same task statement and same acceptance criteria
- [ ] Run on `small` dataset until accepted correctness
- [ ] Run benchmark on `medium`
- [ ] Run stress pass on `large`
- [ ] Record iterations to accepted output
- [ ] Record total generation/build/run time
- [ ] Record regressions introduced during refinement
- [ ] Record final acceptance quality (0/1)
- [ ] Record developer UX score (0..40)

Artifacts:
- [ ] prompt history
- [ ] final source
- [ ] output files
- [ ] metrics summary

## 4) Reproducibility Pass

- [ ] Run SCULPT reproducibility reruns (N=5)
- [ ] Run vibe reproducibility reruns (N=5)
- [ ] Compute output hashes
- [ ] Count unique hashes per side
- [ ] Convert to reproducibility score used in gate

## 5) Gate Fill + Check

- [ ] Copy template gate to `poc/gates/data_heavy_vibe_gate.json`
- [ ] Fill all measured values
- [ ] Run: `sculpt gate check poc/gates/data_heavy_vibe_gate.json`
- [ ] Store command output in report

## 6) Report Finalization

- [ ] Fill `poc/SCULPT_vs_Vibe_Data_Heavy_Report.md`
- [ ] Add final verdict (`go` / `conditional-go` / `no-go`)
- [ ] Add rationale and concrete follow-up actions
- [ ] Link report from case study overview

## 7) Quality Guardrails

- [ ] No post-hoc metric redefinition
- [ ] No changed acceptance criteria mid-run
- [ ] No asymmetric retries between approaches
- [ ] Any deviation documented explicitly in report
