# SCULPT Backlog

## Recently Completed

1. Language Core Semantics (P1)
2. Contract System as First-Class Concept (P1)
3. Convergence Controls in Language (P1)
4. Release Quality Gates (P1)
5. Team-Scale Structure Enablement (P1)
- Multi-file imports via `import("file.sculpt") [as Alias]`
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

## Validation, Tooling, and Scale

1. Data-Heavy Benchmark Case Study (P1)
- Run a strict SCULPT-vs-vibe case study on a data-heavy task.
- Reuse the same gates/metrics as current workflow-centric benchmark.
- Record go/conditional-go/no-go decision with explicit rationale.

2. Persisted Build Telemetry Expansion (P2)
- Surface normalized timestamps and durations in TUI details.
- Add compact per-run trend view (last N builds) for debugging performance drift.

3. Dist Retention Policy (P2)
- Add retention options to `sculpt clean` (age, count, size budget).
- Optional auto-clean behavior configurable in `sculpt.config.json`.

4. CLI/TUI Regression Coverage (P2)
- Add integration tests for per-script `dist` isolation and run/build behavior parity.
- Add tests for TUI key actions (`Enter`, `B`, `R`, `F`, `P`, `C`) and modal flows.

5. Prompt-Drift Competitive Benchmarking (P2)
- Continuously benchmark SCULPT improvements against prompt-first/vibe baselines.
- Track drift over releases to verify SCULPT remains materially superior in its target category.
