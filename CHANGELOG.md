# Changelog

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
