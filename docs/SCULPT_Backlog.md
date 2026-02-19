# SCULPT Backlog

## Recently Completed

1. Language Core Semantics (P1)
2. Contract System as First-Class Concept (P1)
3. Convergence Controls in Language (P1)
4. Release Quality Gates (P1)

## Success-Critical Foundations

1. Team-Scale Structure Enablement (P1)
- Strengthen namespace/scope + contract validation as the default path for multi-developer projects.
- Add validations for naming collisions and shadowing behavior in strict modes.
- Document team conventions and migration patterns.

## Validation, Tooling, and Scale

2. Data-Heavy Benchmark Case Study (P1)
- Run a strict SCULPT-vs-vibe case study on a data-heavy task.
- Reuse the same gates/metrics as current workflow-centric benchmark.
- Record go/conditional-go/no-go decision with explicit rationale.

3. Persisted Build Telemetry Expansion (P2)
- Surface normalized timestamps and durations in TUI details.
- Add compact per-run trend view (last N builds) for debugging performance drift.

4. Dist Retention Policy (P2)
- Add retention options to `sculpt clean` (age, count, size budget).
- Optional auto-clean behavior configurable in `sculpt.config.json`.

5. CLI/TUI Regression Coverage (P2)
- Add integration tests for per-script `dist` isolation and run/build behavior parity.
- Add tests for TUI key actions (`Enter`, `B`, `R`, `F`, `P`, `C`) and modal flows.

6. Prompt-Drift Competitive Benchmarking (P2)
- Continuously benchmark SCULPT improvements against prompt-first/vibe baselines.
- Track drift over releases to verify SCULPT remains materially superior in its target category.
