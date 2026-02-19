# SCULPT Backlog

## Success-Critical Foundations

1. Language Core Semantics (P1)
- Define formal `must/should/may` behavior for all core constructs.
- Specify conflict handling for `satisfy(...)`, `when`, `emit`, `run`, and termination behavior.
- Add parser/runtime validation errors mapped to those semantics.

2. Contract System as First-Class Concept (P1)
- Extend `@meta` + target contracts into a typed contract layer.
- Validate required capabilities/extensions before LLM execution.
- Expose contract schema for IDE tooling (syntax help, diagnostics, coloring).

3. Convergence Controls in Language (P1)
- Add explicit ND controls in Sculpt code (`nd_budget`, `confidence`, `max_iterations`, `fallback`).
- Enforce convergence constraints in compile pipeline and provider prompt contract.
- Report convergence metrics in build outputs.

4. Release Quality Gates (P1)
- Introduce fixed release gates: reproducibility, regression count, iteration count, UX score, and acceptance quality.
- Require gates for all case-study-driven claims in docs.
- Fail CI/docs publication if gate evidence is missing.

5. Team-Scale Structure Enablement (P1)
- Strengthen namespace/scope + contract validation as the default path for multi-developer projects.
- Add validations for naming collisions and shadowing behavior in strict modes.
- Document team conventions and migration patterns.

## Validation, Tooling, and Scale

6. Data-Heavy Benchmark Case Study (P1)
- Run a strict SCULPT-vs-vibe case study on a data-heavy task.
- Reuse the same gates/metrics as current workflow-centric benchmark.
- Record go/conditional-go/no-go decision with explicit rationale.

7. Persisted Build Telemetry Expansion (P2)
- Surface normalized timestamps and durations in TUI details.
- Add compact per-run trend view (last N builds) for debugging performance drift.

8. Dist Retention Policy (P2)
- Add retention options to `sculpt clean` (age, count, size budget).
- Optional auto-clean behavior configurable in `sculpt.config.json`.

9. CLI/TUI Regression Coverage (P2)
- Add integration tests for per-script `dist` isolation and run/build behavior parity.
- Add tests for TUI key actions (`Enter`, `B`, `R`, `F`, `P`, `C`) and modal flows.

10. Prompt-Drift Competitive Benchmarking (P2)
- Continuously benchmark SCULPT improvements against prompt-first/vibe baselines.
- Track drift over releases to verify SCULPT remains materially superior in its target category.
