# Benchmark Baselines

(C) 2026 byte5 GmbH

This folder stores optional baseline snapshots used for release-over-release delta reporting.

Recommended file:
- `latest_release_gate_result.json`
  - last accepted output from `scripts/ci_benchmark_release_gate.sh`
  - used as `--previous` reference by `scripts/ci_benchmark_delta_report.sh`

If the file is missing, delta reports still run and mark the previous baseline as `none`.
