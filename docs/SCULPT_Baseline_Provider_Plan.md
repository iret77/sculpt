# SCULPT Baseline Provider Plan

## Goal
Bring built-in providers (`cli`, `gui`, `web`) from demo-grade to practical app-grade while keeping SCULPT strict, contract-driven, and predictable.

## Non-Negotiables
- No implicit magic words in ND constraints.
- Every callable symbol is contract-exported and discoverable (`sculpt target packages/exports`).
- Deterministic runtime path remains stable under build/replay.

## Provider Upgrade Blocks

| Block | Scope | Deliverables | Exit Criteria |
|---|---|---|---|
| B1 | Contract Completeness | Expand `ui`, `input`, and `nd` packages for each target. Add descriptions and export lists for every symbol. | Each target can build at least one non-trivial CRUD/business example and one interactive example without undefined symbols. |
| B2 | Deterministic Data Core | Add a shared `data` package profile (`readCsv`, `readJson`, `filter`, `group`, `sort`, `writeCsv`, `writeJson`) with strict signatures. | Data-heavy benchmark artifacts are reproducible and schema-validated. |
| B3 | Practical UI Kit | `cli`: forms/tables/pagination; `gui`: dialogs/forms/list+detail; `web`: board/list/filter/detail patterns. | Real app examples (ticketing/invoice/review) compile without ad-hoc custom glue. |
| B4 | ND Catalogs by Domain | Curated ND vocab sets (`layout`, `tone`, `playability`, `accessibility`, `ops`) exported via `nd` namespace per target. | Examples no longer use unqualified ND names; all ND constraints resolve via contract or local `define`. |
| B5 | Cross-Platform Runtime Parity | `gui` backend parity (macOS/Windows/Linux), equivalent interaction semantics where possible. | Same SCULPT script yields functionally equivalent behavior across desktop platforms. |
| B6 | Quality Gates + Benchmarks | Contract validation tests, provider conformance tests, benchmark regression checks in CI. | Release gate blocks on contract drift, missing exports, benchmark regressions. |

## Target-Specific Focus

### CLI
- Strengths to amplify: deterministic automation/data flows, terminal-first ops tools.
- Missing today: richer widgets, stronger multi-step interaction primitives.
- Priority: B2 -> B3 -> B4.

### GUI
- Strengths to amplify: native-feeling desktop workflows.
- Missing today: parity and robust component coverage.
- Priority: B3 -> B5 -> B6.

### Web
- Strengths to amplify: app-level flows, stack profile abstraction.
- Missing today: deeper practical patterns for dashboard/business apps.
- Priority: B3 -> B4 -> B6.

## Delivery Rhythm (Suggested)
- Sprint 1: B1 + B4 (strict symbol baseline, no ND magic leakage).
- Sprint 2: B2 + CLI half of B3.
- Sprint 3: GUI/Web half of B3 + start B5.
- Sprint 4: B5 completion + B6 hard release gating.

## Current B3 Reference Examples
- CLI: `/Users/cwendler/Dev/App/sculpt/examples/practical_cli_control_center.sculpt`
- GUI: `/Users/cwendler/Dev/App/sculpt/examples/practical_gui_service_desk.sculpt`
- Web: `/Users/cwendler/Dev/App/sculpt/examples/practical_web_ops_portal.sculpt`

## Definition of Practical
- A developer can build a useful app without hidden provider knowledge.
- Symbol discovery is self-service from CLI + docs.
- Behavior is stable enough for CI and team-scale collaboration.
