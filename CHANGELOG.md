# Changelog

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
