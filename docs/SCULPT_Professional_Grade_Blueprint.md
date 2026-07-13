# SCULPT Professional-Grade Blueprint

(C) 2026 byte5 GmbH

> This document records detailed requirements from the current architecture.
> The [SCULPT Convergent Programming Concept](SCULPT_Convergent_Programming_Concept.md)
> is the canonical strategic direction. Where the documents conflict, the
> Convergent Programming concept takes precedence.

## Goal
Define the reusable engineering requirements for SCULPT's single authorized
product path: bounded, evidence-carrying changes to audit-intensive Brownfield
business systems.

This is no longer a plan for universal Greenfield generation, commercial games,
or simultaneous multi-target expansion. Items below are implemented only when
they serve the existence gates in the canonical concept.

## Extended Thesis
The baseline is modern spec-driven agentic development with repository context,
tests, policies, locks, and full build-test-repair workflows. SCULPT is useful
only if a typed semantic and evidence graph makes repeated governed changes
safer and cheaper than that baseline.

SCULPT is designed for professional developers maintaining existing systems:
- brownfield-first,
- typed and intent-oriented,
- closed-world and bounded,
- obligation- and evidence-carrying,
- contract- and policy-controlled.

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
- Keep deterministic semantics for types, effects, boundaries, graph
  invalidation, evidence policy, acceptance, and locks.
- Define strict conflict resolution in the semantic validator.
- Treat models as candidate generators, never as acceptance authorities.
- Require exact replay only when all artifacts are content-addressed and
  frozen; otherwise require semantic and evidence equivalence.

## 2) Convergence Control (AI-First At Scale)

### 2.1 Convergence Budgets And Permissions
- Add per-unit budgets for time, tokens, money, iterations, and diff size.
- Define typed freedom slots and default-deny file, effect, tool, secret, and
  network permissions.
- Remove success fallbacks: a stub or unrelated replay may diagnose or unblock
  development, but can never satisfy an accepted build.

### 2.2 Decision Explainability
- Build output must include:
  - which typed freedom slots were exercised,
  - which decisions were recorded,
  - which obligations constrained them,
  - why exact evidence policy accepted or rejected the candidate.

### 2.3 Graph-Scoped Context
- Compute the affected semantic graph slice before synthesis.
- Include only affected declarations, boundaries, obligations, freedom slots,
  contracts, and diagnostics.
- Treat context reduction as a measured result of sound invalidation, not a
  positional compact-IR goal.

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

### 4.2 Obligations And Evidence As Language Features
- Add typed obligations, scenarios, properties, verifier references, and
  evidence policies.
- Bind evidence to exact semantic subjects, inputs, environment, producer, and
  toolchain.
- Distinguish candidate, accepted, and releasable builds.
- Make changed dependencies stale automatically.

### 4.3 Observability
- Build and runtime telemetry standards:
  - provider/model, token and cost, latency, candidate iterations, artifact and
    semantic hashes, invalidation, evidence freshness, and human intervention.
- Export evidence packages that can support an organization's audit process.
- Never claim that SCULPT itself grants compliance or certification.

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

## 6) Brownfield And Provider Architecture

### 6.1 Minimal Semantic Graph + Provider Lowering
- The compiler owns a small graph kernel for types, effects, units, boundaries,
  obligations, freedom slots, decisions, artifacts, and evidence.
- Providers and packages own stack/domain schemas and native lowering.
- There is no universal UI-oriented Target IR.

### 6.2 Ownership Modes
- Native regions remain conventional repository source and are never
  overwritten implicitly.
- Governed regions accept bounded native patches subject to SCULPT obligations.
- Derived regions may be regenerated from SCULPT source and locked decisions.
- Guarantees stop at declared opaque boundaries.

### 6.3 Provider Responsibilities
- Publish versioned capabilities, effects, freedom slots, tools, validators,
  sandbox permissions, diagnostics, and compatibility policy.
- Produce native patches or provider-specific IR as appropriate.
- Contain no application- or benchmark-specific business implementation.
- Pass conformance, second-repository, independent-extension, and maintenance
  cost gates before any ecosystem expansion.

### 6.4 Hotfix Operations
- Detect direct patches against known artifact hashes and mark affected units
  tainted.
- Support time-bounded emergency overrides with explicit human and executable
  waiver policy.
- Require reconciliation into typed source, a native boundary/provider
  extension, or a revert before the next normal release.

## 7) Governance And Release Discipline
- Language spec versioning (`syntax` + `semantics` + `contracts`).
- Compatibility policy (what is breaking, what is additive).
- Feature flags for experimental syntax, never default-on in stable channel.

## 8) Recommended Delivery Phases

### Phase 0 — Commercial Gate
- Provider compatibility corridor frozen before counted interviews.
- Four external Brownfield repositories from two organizations and their change
  sequences inside that corridor.
- One paid pilot or equivalent binding commitment.

### Phase A — Minimal Kernel
- Semantic graph and stable IDs.
- Typed obligations, evidence, freedom slots, effects, and ownership.
- Closed-world policy and candidate/accepted separation.

### Phase B — Vertical Agentic Slice
- One real model, one real failing verifier, one repair loop.
- Native patch, decision ledger, evidence package, and atomic acceptance.

### Phase C — Repeated Change And Operations
- Incremental invalidation and granular lock.
- Typed refinement and explicit relaxation.
- Emergency override and reconciliation.

### Phase D — Provider And Existence Proof
- One first-party design-partner provider.
- Second repository and independent SDK extension.
- Frozen three-arm experiment and independent audit.

Additional targets, a registry, language server expansion, and broader
enterprise tooling are post-Full-Go only.

## Short Answer: Namespaces And Scopes?
Yes, absolutely.
Without namespaces/scopes, large systems become ambiguous, hard to review, and unsafe for multi-team work.
They are mandatory for professional-grade SCULPT.
