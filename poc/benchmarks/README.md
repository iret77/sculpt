# Benchmark Baselines

(C) 2026 byte5 GmbH

This folder stores optional baseline snapshots used for release-over-release delta reporting.

Recommended file:
- `latest_release_gate_result.json`
  - last accepted output from `scripts/ci_benchmark_release_gate.sh`
  - used as `--previous` reference by `scripts/ci_benchmark_delta_report.sh`

Matrix benchmark gate output:
- `poc/tmp/benchmark_matrix_gate_result.json`
  - emitted by `scripts/ci_benchmark_matrix_gate.sh`
  - aggregates pass/fail across:
    - data-heavy competitive gate
    - workflow competitive gate
    - UI practical quality gate

Safety update path:
- `scripts/ci_benchmark_baseline_update.sh`
  - validates that release gate is fully green before writing baseline
  - default mode `candidate` writes:
    `poc/tmp/latest_release_gate_result.candidate.json`
  - `inplace` mode updates:
    `poc/benchmarks/latest_release_gate_result.json`

CLI shortcut:
- `sculpt benchmark baseline update --mode candidate`
- `sculpt benchmark baseline update --mode inplace`

If the file is missing, delta reports still run and mark the previous baseline as `none`.
