# SCULPT Professional-Grade Blueprint

(C) 2026 byte5 GmbH

## Goal
Define what SCULPT needs to support real-world, large-scale software projects
(ERP, commercial games, multi-team platforms) with predictable quality.

## Extended Thesis
At scale, fully hand-written code is no longer the expected default workflow.
AI-assisted development is becoming standard, but pure prompting is weak in structure, reproducibility, and team governance.

SCULPT is designed as a stronger model for developers who think in code:
- intent-oriented,
- AI-native,
- convergent,
- contract- and policy-ready for production environments.

## 1) Language Foundations

### 1.0 Domain-First Structure (DDD-light)
- Organize modules by business/game domain, not by technical utility folders.
- Use bounded-context style roots (`Billing`, `Inventory`, `Gameplay`, `UI`).
- Keep this lightweight: naming and contracts first, process-heavy DDD patterns optional.

### 1.1 Module System
- Add `package(...)`, `import ...`, `export ...`.
- Enforce explicit dependency graph (no implicit cross-module access).
- Detect and reject cyclic imports unless explicitly allowed by rule.

### 1.2 Namespaces And Scopes (mandatory)
- Introduce lexical scopes for variables (module, flow, state, rule, local block).
- Introduce fully-qualified names for symbols:
  - `package.module.flow.state`
- Reject duplicate names inside same scope.
- Shadowing rules must be explicit and deterministic.

### 1.3 Strong Type Layer
- Add first-class `type(...)`, `enum(...)`, `entity(...)`.
- Add typed state declarations and function signatures.
- Enforce compile-time type checks before LLM compile step.

### 1.4 Contract-First Capabilities
- Convert provider contracts into typed language-level contracts:
  - `contract(...)`, `requires(...)`, `provides(...)`
- Compile must fail if required capability is missing for selected target.

### 1.5 Deterministic Core Semantics
- Keep `on/when/emit/run/terminate` execution order deterministic.
- Define strict conflict resolution in semantic validator.
- Require replayable builds in CI for all release branches.

## 2) Convergence Control (AI-First At Scale)

### 2.1 ND Budgets
- Add domain/module-level budgets:
  - `nd_budget`, `confidence`, `max_iterations`, `fallback`.
- Default strictness profiles by domain:
  - `finance`: very low ND
  - `ui`: medium ND
  - `content`: higher ND

### 2.2 ND Explainability
- Build output must include:
  - what was non-deterministic,
  - which constraints reduced solution space,
  - why final result was accepted.

### 2.3 Contract-Scoped Prompt Compression
- Precompile Sculpt + relevant contract subset into compact IR.
- Include only used capabilities/extensions to reduce context size.

## 3) Team-Scale Collaboration

### 3.1 Ownership Model
- Add metadata:
  - `@owner`, `@reviewers`, `@criticality`.
- CI must verify owner approvals for critical modules.

### 3.2 API/Contract Versioning
- Mandatory semantic versioning for exported contracts.
- Deprecation windows and migration markers required.

### 3.3 Workspace/Monorepo Support
- Multi-module workspace config.
- Incremental builds by dependency graph.
- Impact analysis for changes (who breaks if contract changes).

## 4) Quality, Safety, And Compliance

### 4.1 Policy Engine
- Add policy layer for:
  - security constraints,
  - data/privacy constraints,
  - runtime/resource constraints.
- Compile fails on policy violations.

### 4.2 Testing As Language Feature
- Add `spec(...)`, `scenario(...)`, `property(...)`.
- Replay mode required in CI for deterministic verification.

### 4.3 Observability
- Build and runtime telemetry standards:
  - provider/model, token usage, latency, ND metrics, artifact hash.
- Audit logs for regulated business domains.

## 5) Compiler/Tooling Requirements

### 5.1 Semantic Validator Layer
- Dedicated validator stage after parse and before LLM.
- Stable diagnostic codes and machine-readable output.

### 5.2 IDE Integration
- Language server with:
  - completion,
  - type diagnostics,
  - contract diagnostics,
  - scope/name navigation.

### 5.3 Artifact Management
- Per-script output isolation (already done).
- Retention policies and cleanup automation.

## 6) Runtime/Target Architecture

### 6.1 Standard Target IR + Extensions
- Keep standard target IR as stable baseline.
- Allow target-specific extension fields only via declared contracts.

### 6.2 Build Provider Responsibilities
- Pre-LLM: publish capability/contract schema.
- Post-LLM: validate generated target IR, build deterministic artifact, return run descriptor.

### 6.3 Run Orchestration
- `sculpt run` remains single interface.
- Provider decides execution details; developer does not manage platform tooling directly.

## 7) Governance And Release Discipline
- Language spec versioning (`syntax` + `semantics` + `contracts`).
- Compatibility policy (what is breaking, what is additive).
- Feature flags for experimental syntax, never default-on in stable channel.

## 8) Recommended Delivery Phases

### Phase A (now)
- Semantic validator V1
- Namespace/scope rules V1
- Contract typing V1

### Phase B
- ND budget controls + reporting
- Workspace dependency graph
- CI replay gates

### Phase C
- Type system expansion
- Language server
- Policy engine

### Phase D
- Full versioned contract ecosystem
- Enterprise compliance/audit tooling
- Multi-target release orchestration

## Short Answer: Namespaces And Scopes?
Yes, absolutely.
Without namespaces/scopes, large systems become ambiguous, hard to review, and unsafe for multi-team work.
They are mandatory for professional-grade SCULPT.
