# SCULPT Reference Provider Plan

Status: **replaces the former three-target baseline provider program as of
2026-07-12**.

The legacy filename is retained to avoid broken links. The built-in CLI, GUI,
and Web providers are maintenance-only. This plan authorizes one first-party
Brownfield reference provider after the commercial gate in the
[canonical concept](SCULPT_Convergent_Programming_Concept.md) passes.

## Goal

Prove that the SCULPT kernel can govern repeated native changes in a real
design-partner stack without compiler-owned domain logic and with commercially
viable setup and maintenance cost.

Before the first counted commercial interview, the project freezes a provider
compatibility corridor: language/runtime, supported framework-major range,
build/test interface, data boundary, and deployment shape. G0 requires four
real repositories from two external organizations inside that corridor. The
selection is not driven by existing demos and remains fixed through the
existence gate.

## Architecture Boundary

| Layer | Responsibility |
|---|---|
| Compiler kernel | Typed graph, permissions, invalidation, convergence, evidence policy, locks |
| Stack adapter | Native build/test/run/package tools, diagnostics, sandbox declarations |
| Capability packs | Versioned stack and domain contracts, effects, freedom slots |
| Organization assurance | Policies, validators, human authority, external trust roots |

Model, stack, and assurance providers remain logically separate even when one
bundle implements multiple roles.

## Required Deliverables

| Block | Deliverable | Exit criterion |
|---|---|---|
| R1 | Versioned provider protocol | Capabilities, effects, freedom slots, tools, diagnostics, validators, sandbox permissions, and compatibility rules validate before synthesis. |
| R2 | Native Brownfield binding | Existing types, schemas, APIs, tests, and owned regions enter the graph without a full rewrite. |
| R3 | Bounded patch transaction | Agent can edit only declared regions and the provider returns attributable native patches. |
| R4 | Verification integration | Native static checks, builds, tests, and external validators emit evidence attestations for exact semantic subjects. |
| R5 | Operations integration | Emergency override, taint, waiver, reconciliation, and full reverification work on the real stack. |
| R6 | Conformance suite | Clean/incremental equivalence, permission denial, provenance, evidence freshness, and compatibility are machine-tested. |
| R7 | SDK extension proof | An independent developer adds a non-trivial capability without compiler changes or fork. |
| R8 | Second-repository proof | Same provider supports a second real repository within the fixed setup and maintenance limits. |

## Hard Prohibitions

- No claims, invoices, games, showcase, benchmark, or other application
  behavior in compiler or stack adapter.
- No unknown symbol or prose phrase interpreted as an implicit capability.
- No provider self-certification as the sole evidence for its own critical
  output.
- No fallback stub or replay accepted as evidence of real synthesis.
- No provider override of core graph, invalidation, or acceptance semantics.
- No broad catalog built in anticipation of hypothetical applications.
- No second target, registry, or marketplace before Full-Go.

## Portability And Economics Gates

The provider passes only if:

- the second real repository needs no compiler change and at most five
  person-days of project-specific adapter work;
- a representative minor stack upgrade needs at most two person-days;
- the independent SDK extension passes the same conformance suite;
- provider authoring, setup, support, upgrades, and failed attempts appear in
  the fully loaded benchmark cost;
- native output remains buildable and maintainable without SCULPT.

Application-specific business contracts remain customer-owned. A later
commercial hypothesis is paid LTS provider bundles, assurance integrations,
and support. It is not a current product claim.

## Existing Providers

The existing CLI, GUI, and Web providers remain useful prototype and regression
fixtures. They do not satisfy this plan, and their demo-specific behavior must
not enter the reference proof.

Allowed work before Full-Go is limited to security, critical regression fixes,
and reuse directly required by the single reference-provider path.
