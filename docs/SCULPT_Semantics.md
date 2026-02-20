# SCULPT Core Semantics (Draft v0.1)

(C) 2026 byte5 GmbH

## Purpose
This document defines **normative language behavior** for SCULPT core constructs.
It is the semantic baseline for parser validation, IR generation, LLM compile contracts, and target builds.

Normative terms:
- **MUST**: required behavior
- **SHOULD**: recommended behavior
- **MAY**: optional behavior

## 1. File And Root Rules
- A program **MUST** contain exactly one `module(<Identifier>) ... end`.
- `module(...)` **MAY** be dot-qualified for namespace paths (e.g. `module(Billing.Account.Invoice)`).
- Content outside the module (except comments and `@meta`) **MUST NOT** exist.
- Unknown top-level blocks **MUST** fail validation.

## 2. Core Construct Semantics

### 2.1 `module(name)`
- The module is the root scope.
- Names **MUST** be valid identifiers.
- Dot-qualified namespace segments **MUST** each be valid identifiers.
- Module name **SHOULD** be stable across versions of the same app.

Strict scope mode:
- `@meta strict_scopes=true` enables shadowing checks (`NS505`).

### 2.2 `flow(name)`
- A flow defines a directed state graph.
- A flow **MUST** define exactly one `start > <StateName>`.
- The start target **MUST** resolve to a state declared inside that flow.
- Flow names **MUST** be unique within a module.

### 2.3 `state(name)` and `state()`
- `state(name)` defines a flow-local state.
- `state()` defines global state storage (module scope).
- A named state **MUST** belong to exactly one flow.
- State names **MUST** be unique per flow.
- Multiple `state()` blocks **MAY** exist; they are merged in source order.

### 2.4 `rule(name)`
- Rules are deterministic transformations over state/events.
- A rule body **MUST** contain at least one trigger (`on ...` or `when ...`).
- Rule names **MUST** be unique within a module.
- `emit <event>` is only valid inside rules.

### 2.5 `nd(name, ...)`
- ND blocks define non-deterministic solution space for LLM compilation.
- An ND block **MUST** contain at least one `propose ...` statement.
- An ND block **MUST** contain one `satisfy(...)` block with at least one constraint.
- `propose` expands candidate space; `satisfy` constrains it.

### 2.6 `run <FlowName>`
- `run` invokes a flow from inside a state.
- Target flow **MUST** exist.
- If used, the parent state **SHOULD** define how control returns (commonly via `on done > ...`).

### 2.7 `terminate`
- `terminate` marks successful program stop.
- When reached, execution **MUST** stop immediately.

## 3. Event And Transition Model
- `on <eventCall> > <StateName>` registers a transition for the current state.
- Transition targets **MUST** resolve to states in the same flow.
- For one state, the same event signature **MUST NOT** map to multiple targets.
- Runtime transition selection **MUST** be deterministic.

Recommended deterministic rule:
1. Exact event-signature match.
2. First declaration order if multiple handlers are still equivalent.

## 4. Conflict Resolution

### 4.1 Rule Trigger Conflicts
- If multiple rules are triggered in one cycle, they execute in source order.
- State updates from earlier rules **MUST** be visible to later rules in the same cycle.

### 4.2 `when` vs `on`
- `on` is event-driven.
- `when` is condition-driven.
- Supported `when` operators: `>=`, `>`, `<`, `==`, `!=` and logic chaining with `and` / `or`.
- If both trigger in a cycle, rule order is still source order (single deterministic ordering).

### 4.3 `emit` Ordering
- Emitted events are queued FIFO within the current cycle.
- Consumers process emitted events deterministically in queue order.

### 4.4 `satisfy(...)` Conflicts
- All constraints in `satisfy(...)` are hard constraints.
- Duplicate constraint calls in one `satisfy(...)` list **MUST** fail semantic validation.
- If constraint set is unsatisfiable, compile **MUST** fail with ND constraint error.

### 4.5 `run` and `terminate`
- `terminate` has highest priority and ends execution immediately.
- If both a transition and terminate become active in the same cycle, terminate wins.

## 5. Validation Error Contract (Compiler)

## 5.1 Structural
- `S001` Missing module root.
- `S002` Multiple module roots.
- `S003` Unexpected top-level construct.
- `S004` Unbalanced or missing `end`.
- `S005` Identifier expected.

## 5.2 Flow/State
- `F101` Duplicate flow name.
- `F102` Missing `start` in flow.
- `F103` Unknown start state.
- `F104` Duplicate state name in flow.
- `F105` Unknown transition target state.
- `F106` Duplicate event handler signature in one state.

## 5.3 Rules
- `R201` Duplicate rule name.
- `R202` Rule has no effect body.
- `R203` `emit` used outside rule.
- `R204` `when` expression is not a supported comparison form.
- `R205` `emit` event name is invalid.

## 5.4 ND
- `N301` ND block has no `propose`.
- `N302` ND block has no `satisfy`.
- `N303` Empty `satisfy(...)`.
- `N304` Duplicate/invalid ND constraint set in `satisfy(...)`.
- `N305` `nd_budget=0` used while ND blocks exist.
- `N306` ND magicword (`?value`) used while `nd_policy=strict`.

## 5.5 Runtime/Binding
- `B401` `run` references unknown flow.
- `B402` Invalid `terminate` placement.
- `B403` Multiple `run` targets in one state.
- `B404` `run` without explicit `on done > ...` return path.

## 5.6 Namespace/Scope
- `NS501` Invalid namespace segment.
- `NS502` Duplicate namespace symbol in same scope.
- `NS503` Unknown qualified name reference.
- `NS504` Illegal cross-namespace reference without contract/import.
- `NS505` Forbidden shadowing in strict mode.
- `NS506` Ambiguous unqualified symbol reference.

## 5.7 Meta/Convergence Controls
- `M705` Invalid `nd_policy` (must be one of `strict|magic`).
- `M701` Invalid `nd_budget` (must be integer `0..100`).
- `M702` Invalid `confidence` (must be number `0.0..1.0`).
- `M703` Invalid `max_iterations` (must be integer `1..10000`).
- `M704` Invalid `fallback` (must be one of `fail|stub|replay`).

## 5.8 Target Contract Validation
- `C901` Invalid `@meta` value for declared contract type/range.
- `C902` Required capability missing on selected target contract.
- `C903` Unknown `@meta` key not declared in contract (except `x_` extension keys).
- `C904` `layout=explicit` requested but target lacks `layout.explicit` capability.

## 6. LLM Contract Implications
- Parser + semantic validator produce canonical IR only for valid programs.
- ND constraints are passed as hard requirements in the LLM compile request.
- Target providers build only validated IR output.
- Replay mode must bypass LLM and use locked deterministic artifacts.

## 7. Implementation Plan (Compiler)

### Phase 1: Semantic Validator Layer
- Add `src/semantics/` module.
- Input: AST; Output: diagnostics list (`code`, `message`, `span`, `severity`).
- Implement checks for `S*`, `F*`, `R*`, `N*`, `B*` (except deep unsat detection).

### Phase 2: CLI + UX Integration
- Print structured semantic diagnostics in CLI and TUI log.
- Fail `build/freeze/replay/run` early on semantic errors.
- Keep parse errors and semantic errors clearly separated.

### Phase 3: Deterministic Execution Ordering
- Encode source-order rule scheduling explicitly in IR/runtime metadata.
- Ensure generated target code preserves this order.

### Phase 4: ND Constraint Enforcement
- Add provider-side and post-LLM validation hooks for `satisfy(...)`.
- Return `N304` when generated output violates hard constraints.

### Phase 5: Tests
- Add `tests/semantics_test.rs` with positive + negative cases.
- Add regression tests for ordering, duplicate handlers, unknown targets, and ND missing parts.
