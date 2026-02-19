# SCULPT Backlog

## Near Term

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

## Compiler & Tooling

4. Persisted Build Telemetry Expansion (P2)
- Extend `dist/<script>/build.meta.json` with token usage per provider when available.
- Surface normalized timestamps and durations in TUI details.

5. Dist Retention Policy (P2)
- Add retention options to `sculpt clean` (age, count, size budget).
- Optional auto-clean behavior configurable in `sculpt.config.json`.

6. CLI/TUI Regression Coverage (P2)
- Add integration tests for per-script `dist` isolation and run/build behavior parity.
- Add tests for TUI key actions (`Enter`, `B`, `R`, `F`, `P`, `C`) and modal flows.
