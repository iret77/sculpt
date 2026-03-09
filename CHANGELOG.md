# Changelog

## 0.2.21
- Closed Milestone D with non-demo practical target quality gates in CI:
  - `.github/workflows/target-practical-gates.yml`
  - `scripts/ci_target_practical_gates.sh`
- Added practical scenario validation for all built-in targets:
  - CLI: `examples/practical/cli_control_center.sculpt`
  - GUI: `examples/practical/gui_service_desk.sculpt`
  - Web: `examples/practical/web_ops_portal.sculpt`
- Practical gates now enforce both:
  - source-level complexity checks on `ir.json` (state/transition thresholds)
  - deterministic target runtime/artifact checks (`target.ir.json`, runtime hooks, and required output files).
- Updated roadmap/backlog milestone status for D completion.

## 0.2.20
- Added CLI baseline-update command:
  `sculpt benchmark baseline update --mode candidate|inplace`
- Command enforces the same pass-only baseline guards as the CI helper script:
  gate pass, zero failures, criteria pass, acceptance/repro thresholds.

## 0.2.19
- Added pass-only benchmark baseline update guard:
  `scripts/ci_benchmark_baseline_update.sh`
- Updated benchmark release-gate workflow to publish baseline candidate artifact:
  `poc/tmp/latest_release_gate_result.candidate.json`
- Updated benchmark docs/backlog with guarded baseline update flow (`candidate` vs `inplace`).

## 0.2.18
- Initialized benchmark release baseline snapshot:
  `poc/benchmarks/latest_release_gate_result.json`
- Delta reporting can now compare against a concrete previous baseline by default.

## 0.2.17
- Added automated benchmark delta reporting scripts:
  `scripts/ci_benchmark_delta_report.sh` and `scripts/benchmark_delta_report.py`
- Updated benchmark release-gate workflow to publish machine-readable artifacts:
  release gate metrics/result + delta report JSON/Markdown.
- Added baseline benchmark documentation in `poc/benchmarks/README.md`.

## 0.2.16
- Added competitive benchmark release gate automation:
  `scripts/ci_benchmark_release_gate.sh`
- Added CI workflow for the release benchmark gate:
  `.github/workflows/benchmark-release-gate.yml`
- Release gate now enforces:
  fresh SCULPT benchmark + SCULPT gate check + SCULPT-vs-vibe competitive criteria.
- Updated backlog and POC index documentation for the new release-gate path.

## 0.2.15
- Added web adapter quality CI workflow:
  `.github/workflows/web-adapter-quality.yml` + `scripts/ci_web_adapter_quality.sh`
- Added profile-aware web quality gates for `standard`, `next-app`, and `laravel-mvc`:
  adapter registry validation, metadata checks, and runtime hook checks.
- Updated backlog and web target reference with the new quality gate path.

## 0.2.14
- Added GUI widget parity baseline v1 in generated runtimes:
  `heading`, `input`, `checkbox`, `table`, `panel`, `card`, `modal`.
- Extended GUI parity smoke checks to assert widget-render branch coverage in generated runtime source.
- Updated GUI target reference and backlog progress for parity tracking.

## 0.2.13
- Added GUI runtime state-machine parity v1:
  generated GUI runtimes now apply `flow.start` and `flow.transitions` dynamically.
- Updated macOS SwiftUI GUI emitter with runtime key dispatch (`KeyCapture`) and state transition handling.
- Updated Tkinter GUI emitter to runtime transition dispatch (Enter/Esc/keys + click event path).
- Strengthened GUI parity smoke checks for runtime transition markers.

## 0.2.12
- Added GUI shared interaction contract v1 in generated runtimes:
  - `Enter` triggers the primary action
  - `Esc` closes the active window
- Hardened GUI parity smoke checks to validate interaction markers in generated runtime source.
- Updated GUI target reference and backlog progress for parity milestone tracking.

## 0.2.11
- Added benchmark rerun Go/No-Go gate criteria to backlog documentation.
- Added deterministic target quality gate CI workflow:
  `.github/workflows/target-quality-gates.yml` + `scripts/ci_target_quality_gates.sh`
- Added behavior-level output checks for `cli/gui/web` smoke targets:
  runtime source markers plus target IR shape validation.

## 0.2.10
- Added cross-platform GUI parity CI workflow:
  `.github/workflows/gui-parity.yml` + `scripts/ci_gui_parity.sh`
- Added web profile CI smoke workflow:
  `.github/workflows/web-profiles-smoke.yml` + `scripts/ci_web_profiles_smoke.sh`
- Added baseline web standard-profile example:
  `examples/web/web_profile_standard.sculpt`

## 0.2.9
- Added target smoke CI pipeline:
  `.github/workflows/target-smoke.yml` + `scripts/ci_target_smoke.sh`
- Added baseline target artifact checks for `cli`, `gui`, and `web` in CI:
  each smoke build must emit `dist/<script>/target.ir.json`

## 0.2.8
- Added CI provider gate pipeline:
  `.github/workflows/provider-gates.yml` + `scripts/ci_provider_gates.sh`
- CI now checks:
  provider conformance matrix + deterministic data-heavy smoke benchmark + gate validation
- Added smoke-threshold normalization in CI gate script to align gate checks with `--repro-runs 1`

## 0.2.7
- Added provider conformance matrix command: `sculpt auth conformance` (`--providers`, `--verify`, `--json`)
- Added normalized provider telemetry fields in `build.meta.json`:
  `requested_provider`, `requested_model`, `strict_provider`, `fallback_mode`
- Added optional `@meta contract_version` compatibility checks against active target contract version
- Expanded target symbol signature catalog output in `sculpt target exports` across `ui/input/window/net/data/guide`
- Hardened data-heavy benchmark path: namespaced data writer/runtime ops now pass deterministic required-output checks

## 0.2.2
- Added `sculpt project create` with `-p/--path` and `-f/--files` (glob support)
- Added CLI and TUI progress bars in the byte5 palette for build pipeline visibility
- Expanded beginner documentation with ELI5 explanations and runnable starter paths
- Added README elevator-pitch refresh and user-story based start navigation
- Added version-bump guard script and CI workflow to enforce version discipline

## 0.2.0
- Added explicit SCULPT versioning policy (language vs. component versioning)
- Compiler now prints supported language range in key CLI outputs
- Added language support visibility in TUI startup log
- Documentation moved from `v0.1` wording to `Language 1.0` wording

## 0.1.0
- Initial compiler release
- Parser + AST for SCULPT syntax
- AI-backed build pipeline with target IR
- Built-in targets: cli, gui, web
- Example programs and syntax manifest
