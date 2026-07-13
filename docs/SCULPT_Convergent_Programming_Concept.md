# SCULPT Convergent Programming Concept

(C) 2026 byte5 GmbH

Status: **binding strategic reset and existence test**, 2026-07-12.

This document is the canonical strategic direction for SCULPT. Where older
roadmaps, target plans, examples, or architecture documents conflict with it,
this document takes precedence.

## 0. Decision

**GO for one bounded existence program. NO-GO for the previous universal
Greenfield and multi-target direction.**

SCULPT will be pursued only as a brownfield-first, evidence-carrying change
compiler for professional teams that must change consequential business
systems repeatedly and prove why each change is acceptable.

The following previous ambitions are retired as product theses:

- a universal application-generation language,
- a default coding interface for all developers,
- programming for non-programmers,
- simultaneous CLI, GUI, Web, game, and future-target expansion,
- benchmark wins against prompt-only or vibe-coding baselines,
- a provider marketplace before product-market fit.

The current compiler is an engineering foundation and research prototype, not
evidence that the new product exists. Full product investment is permitted only
if the fixed gates in this document pass. If any existential gate fails after
the single allowed remediation cycle, SCULPT is archived. The thresholds,
competitor, or target market must not be changed to rescue the result.

A concept review can validate that this is a coherent investment thesis. Only
measured technical and commercial evidence can validate its existence.

## 1. Core Thesis

**SCULPT is a programming language and compiler for bounded, evidence-carrying
changes to existing software.**

A SCULPT module declares:

- the native system and semantic units a change may affect,
- typed inputs, outputs, effects, and external boundaries,
- what must remain true,
- which decisions an AI system is explicitly allowed to make,
- which decisions are forbidden or require human authority,
- how every mandatory obligation must be evidenced,
- which accepted decisions and evidence remain valid after later changes.

SCULPT compiles that intent into a bounded native-code candidate, executes the
required verification, and emits a patch, provenance, decision ledger, and
evidence package. A model is an **untrusted search worker** inside the compiler,
not an acceptance authority. Only the deterministic compiler kernel and the
declared evidence policy may accept a candidate.

This model is called **Convergent Programming**: compile-time search over a
typed solution space, followed by deterministic acceptance. It does not imply
runtime randomness. A released implementation may be entirely deterministic.

SCULPT does not replace TypeScript, Java, Rust, SQL, or other implementation
languages. It governs selected changes and units while ordinary source remains
ordinary source.

## 2. The Real Competitor

The competitor is not a developer typing one vague prompt. Modern coding agents
already inspect repositories, follow persistent instructions, plan, edit,
build, test, repair, use tools, and work from structured specifications.
Spec-driven toolkits already provide versioned specifications, staged
refinement, task decomposition, agent independence, loops, and human
checkpoints.

SCULPT must therefore beat the strongest practical alternative:

> a current coding agent operating over an existing repository with structured
> specifications, typed APIs, acceptance tests, policies, hooks, CI, lockfiles,
> persistent project instructions, and full build-test-repair rights.

| Best-practice agentic development already provides | SCULPT must add materially |
|---|---|
| Versioned specifications and plans | Compiler-checked symbols, types, effects, boundaries, and stable obligation identities |
| Persistent repository instructions | Closed-world, machine-enforced change and decision permissions |
| Build-test-repair loops | Obligation-specific acceptance controlled outside the synthesis agent |
| Tests and CI gates | Evidence records bound to exact semantic subjects and automatically made stale by relevant changes |
| Lockfiles and reproducible environments | Granular decision, artifact, provider, and evidence locks with semantic invalidation |
| Agent-generated summaries | Source-to-obligation-to-decision-to-artifact-to-evidence traceability |
| Manual impact review | Typed dependency and impact analysis with safe rename and selective recompilation |
| Spec refinement | Typed, auditable adoption of recorded implementation decisions back into the solution space |

## 3. Why A Language At All

The cheapest counter-hypothesis is not prompt-first development. It is:

> structured Markdown or schemas + tests + policies + lockfiles + a capable
> coding-agent harness in the native repository.

That counter-hypothesis is presumed superior until SCULPT disproves it.

SCULPT earns the cost of a language only if its compiler can answer, reliably
and more economically:

1. Which typed obligations are affected by this semantic change?
2. Which prior evidence and accepted decisions are now stale?
3. Which source regions, effects, tools, secrets, and network operations may an
   agent use for this unit?
4. Which graph slices require recheck, resynthesis, relowering, rebuild, or
   reverification?
5. Why is this exact artifact releasable, and can that answer be reproduced?
6. Which recorded implementation decisions can be adopted into source without
   silently broadening the solution space?

SCULPT is a language because it has names, types, scopes, effects, contracts,
operational acceptance semantics, and a compiler. Its syntax is not the moat.
YAML or Markdown with an equivalent typed graph and compiler would be
functionally the same idea.

If the convention-based baseline can provide the same guarantees and economics
without building an equivalent semantic and evidence graph, SCULPT has no
reason to exist. If the baseline must build that graph, it has reproduced the
SCULPT kernel regardless of surface syntax.

## 4. Market Wedge And Non-Goals

### 4.1 Primary User And Buyer

SCULPT is for professional developers maintaining long-lived systems with:

- frequent business-rule and workflow changes,
- expensive regression, review, approval, or audit work,
- explicit ownership and separation-of-duty requirements,
- enough repeated change volume to amortize semantic modeling and provider
  setup.

The first market hypothesis is audit-intensive back-office software in
insurance, financial operations, healthcare administration, and public-sector
workflows. Software houses delivering repeated variants of such systems are a
plausible early-adopter class.

The user is a developer. The economic buyer may be responsible for engineering,
quality, risk, or compliance. SCULPT's durable value proposition is lower cost
and risk for **change assurance**, not better first-shot code generation.

### 4.2 Initial Product Wedge

The first product outcome is deliberately narrow:

> A team expresses a bounded change to an existing business system as typed
> intent. SCULPT produces a reviewable native patch and a fresh evidence
> package. The release gate refuses the change unless every critical obligation
> is satisfied by an allowed combination of evidence.

### 4.3 Explicit Non-Goals

Until the existence gates pass, SCULPT is not:

- a replacement for native source languages,
- a general-purpose app builder,
- a low-code tool for non-programmers,
- a game engine or UI portability layer,
- a certification authority,
- a claim that tests or evidence equal truth,
- a marketplace or community-provider project,
- a safety-critical product for medical-device, automotive, or aviation
  certification.

SCULPT may support an organization's quality or audit process. It must never
claim that its evidence package itself grants regulatory compliance or
certification.

## 5. Minimal Semantic Kernel

The core graph must remain deliberately smaller than any target platform. It is
not a second universal Target IR or a world model.

### 5.1 Core Node Types

The compiler owns only generic semantic categories:

- declarations, symbols, types, and effects,
- modules, units, and external boundaries,
- capabilities and contracts,
- obligations, policies, and verifiers,
- typed freedom slots and preferences,
- decisions and overrides,
- artifact units and evidence attestations.

Versioned packages and providers extend these categories with domain and stack
schemas. The compiler core must not know React widgets, invoice reconciliation,
GUI controls, or any benchmark-specific operation.

### 5.2 Core Edge Types

The graph uses typed relationships such as:

- imports, calls, reads, writes, and transitions,
- requires, allows, constrains, and owns,
- implements, lowers-to, and generated-from,
- depends-on, evidenced-by, and invalidates.

Every persistent semantic element has a stable ID independent of file position
and safe rename. Every relevant node carries separate interface, semantic, and,
where applicable, output hashes.

### 5.3 Convergence Units

A convergence unit is the smallest independently synthesizable and verifiable
region. Its final syntax remains a language-spec decision; this example is
conceptual:

~~~sculpt
module(Claims.HighValueApproval):
  use(stack.web)
  use(policy.financial_change)

  converge(AddDualApproval) -> native.patch:
    bind existing(Claims.ApprovalService)
    own modify("src/claims/approval/**")

    require obligation(authz):
      claim only_roles(Lead, Compliance)
      evidence static(authz_policy) + executable(unauthorized_access)
    end

    require obligation(dual_control):
      claim distinct_approvers(minimum: 2)
      evidence executable(approval_scenarios) + human(risk_owner)
    end

    preserve public_api, audit_history
    allow decision(ui_layout), decision(microcopy)
    deny network, schema_drop, test_weakening

    verify scenario(approve_high_value_claim)
    verify scenario(reject_self_approval)
  end
end
~~~

Every unit has:

- typed inputs, outputs, effects, and external boundaries,
- explicitly owned source or artifact regions,
- mandatory obligations with stable IDs,
- preferences that can never override obligations,
- typed freedom slots,
- explicit verifiers and evidence policies,
- time, cost, token, iteration, and diff-size budgets.

The permission model is closed-world: an unspecified decision, effect,
capability, file region, tool, secret, or network operation is forbidden. An
agent may not invent symbols or reinterpret an unknown word as intent.

## 6. Compiler Trust Model

SCULPT separates a deterministic control plane from an untrusted candidate
plane.

~~~mermaid
flowchart LR
  A["SCULPT + native repository"] --> B["Typed semantic graph"]
  B --> C["Impact + invalidation"]
  C --> D["Bounded change plan"]
  D --> E["Untrusted synthesis agent"]
  E --> F["Isolated native patch"]
  F --> G["Build + verifier providers"]
  G --> H["Evidence policy engine"]
  H -->|"failed or stale"| I["Bounded diagnostics"]
  I --> E
  H -->|"all mandatory obligations satisfied"| J["Accepted patch + evidence + lock"]
~~~

### 6.1 Deterministic Control Plane

The compiler kernel alone controls:

- parsing, typing, stable IDs, effects, and boundaries,
- graph slicing and invalidation,
- candidate permissions and budgets,
- evidence-policy evaluation,
- acceptance state,
- provenance, locks, and replay.

Models, target providers, and assurance providers may return data and
attestations. They may not redefine core semantics or acceptance policy.

### 6.2 Agentic Compilation Loop

For each affected unit the compiler:

1. parses and type-checks source, contracts, capabilities, and policy;
2. computes the minimal affected graph slice;
3. creates a structured change plan containing allowed files, tools, effects,
   freedom slots, and obligation mappings;
4. opens an isolated workspace transaction;
5. lets the synthesis agent create or patch only allowed native units;
6. invokes provider-owned builds, static checks, tests, and validators;
7. binds results to obligations as evidence attestations;
8. returns only bounded diagnostics and the relevant slice for repair;
9. repeats while measurable progress occurs and all budgets remain available;
10. atomically accepts the patch only after policy satisfaction.

Repeated candidate hashes detect loops. A failure is precise and resumable; it
is never disguised as success through a stub or fallback.

During a candidate loop, the synthesis agent may not weaken obligations,
policies, verification scenarios, acceptance thresholds, or its own
permissions. It may propose such a source change in a separate, non-accepted
change set that requires the declared owner.

Provider tools execute in a sandbox with explicit filesystem, network, secret,
process, and tool permissions. This controls agent capability; it does not
claim that the sandbox makes generated software correct.

### 6.3 Candidate Versus Accepted

- A **candidate build** may contain failed, pending, stale, or waived
  obligations. It is clearly non-releasable.
- An **accepted build** has no mandatory obligation in failed, pending, or stale
  state and no waiver forbidden by policy.
- A **releasable build** is an accepted build that also satisfies the
  environment's human-approval, deployment, and operational policies.

The word successful must never be used for a candidate merely because code
compiled or files were produced.

## 7. Obligations And Evidence

An obligation is a typed claim about identified semantic subjects. Evidence is
an attestation that a particular method produced a verdict for exact subject
and input hashes.

### 7.1 Obligation Record

An obligation records at least:

- stable ID and normalized claim,
- subject IDs and criticality,
- mandatory or advisory status,
- owner,
- evidence policy,
- permitted verifier IDs.

### 7.2 Evidence Attestation

An attestation records at least:

- obligation ID, method class, and verdict,
- subject, input, environment, and toolchain hashes,
- producer identity, version, digest, and trust tier,
- run ID, timestamp, provenance, and expiration rule.

For model evaluation it additionally records the exact judge model, protocol
and prompt hash, sampling parameters, calibration-set hash, and threshold. For
human evidence it records identity, role, reason, signature, and validity.

### 7.3 Evidence Methods

Evidence methods are not a universal ranking:

| Method | Examples |
|---|---|
| Static | Type, effect, schema, contract, invariant, or policy analysis |
| Executable | Unit, integration, scenario, property, mutation, benchmark, or build result |
| Evaluated | Pinned and calibrated model evaluation against explicit criteria |
| Human | Signed approval by an authorized role |
| External | Attestation imported from an approved assurance system |

Unverified is not evidence. It is an obligation state. Core states are pending,
satisfied, failed, stale, and waived.

Policies specify allowed evidence combinations, independence requirements,
quorum, validity windows, and waiver rules. Security, financial-integrity, and
data-correctness obligations may never be satisfied solely by model evaluation
or human assertion; they require at least one policy-approved non-model
verification channel. A producer may not be the sole certifier of its own
critical output.

Evidence is bound to semantic subject hashes. A relevant change automatically
makes it stale. An accepted incremental build and a clean build must produce
the same obligation verdicts for the same locked inputs.

Evidence reduces uncertainty; it does not create truth. Bad requirements,
tests, contracts, tools, calibration, or reviewers can produce bad evidence.
The evidence graph must make those dependencies inspectable rather than hide
them behind a green status.

## 8. Brownfield Coexistence

SCULPT adopts existing systems one unit at a time. It never requires a full
rewrite.

Every code region has one ownership mode:

| Mode | Source of truth | SCULPT authority |
|---|---|---|
| Native | Existing repository source | Read and bind through declared contracts; no overwrite |
| Governed | Existing repository source plus SCULPT obligations | Propose bounded patches and require evidence |
| Derived | SCULPT source and locked decisions | Regenerate within an explicitly managed region |

Foreign types, APIs, schemas, and tests enter the graph through versioned
contracts generated or authored from machine-readable sources where possible.
Generated contracts begin untrusted until checked or approved according to
policy.

SCULPT guarantees end at declared boundaries. It must not infer global safety
through opaque native code. Native target code remains buildable, reviewable,
and maintainable if SCULPT is removed; avoiding platform lock-in is an adoption
requirement.

## 9. Production Hotfix And Reconciliation

Operations may patch native or derived output during an incident. The compiler
must support this reality without pretending the divergence did not occur.

1. A direct patch is detected against a known artifact hash.
2. The affected unit is marked tainted and all dependent evidence becomes
   stale.
3. An emergency override records patch, author, incident, base hash, owner,
   expiration, and the executable and human evidence required by waiver policy.
4. The emergency deployment may proceed only through that explicit,
   time-bounded policy. It is not a normal accepted SCULPT build.
5. Before the next normal release, reconciliation must:
   - encode a typed source constraint or recorded decision,
   - move the behavior behind an explicit native boundary or provider extension,
   - or revert the patch.
6. Full reverification removes the taint and override.

SCULPT does not promise to translate an arbitrary diff into correct high-level
intent. Reconciliation is constrained adoption with explicit human authority,
not magical reverse engineering.

## 10. Incrementality, Locks, And Refinement

### 10.1 Invalidation

Graph edges determine whether a change requires recheck, resynthesis,
relowering, rebuild, or reverification. Cache correctness dominates cache hit
rate: conservative extra work is acceptable; a false cache hit is not.

The graph test suite must establish:

- clean-build and incremental-build semantic equivalence,
- 100% invalidation recall for critical obligations,
- safe rename and impact analysis,
- deterministic evidence freshness,
- content-addressed artifact reuse.

Invalidation precision and cache savings remain measured economic outcomes, not
assumed benefits.

### 10.2 Granular Lock

The lock records:

- semantic unit and boundary hashes,
- accepted decisions and freedom-slot versions,
- provider, model, toolchain, policy, verifier, and contract versions,
- evidence attestations and artifact digests,
- context and input digests needed for replay.

Exact byte replay is promised only when every required artifact is
content-addressed and frozen, in which case no model call is needed. Otherwise
SCULPT claims semantic reproducibility against declared obligations, not
byte-identical generation.

### 10.3 Progressive Formalization

Every synthesized choice must reference a typed provider-defined freedom slot
and appear in a decision ledger. A refinement operation may propose selected
ledger decisions as source changes.

Refinement narrows the solution space monotonically. Broadening it requires a
separate, explicit, reviewable relaxation operation. Refinement never silently
turns arbitrary generated code into trusted semantics.

## 11. Provider Model And Economics

The architecture separates four responsibilities:

| Layer | Owner and responsibility |
|---|---|
| Compiler kernel | SCULPT team: graph, policy, invalidation, convergence, evidence, locks |
| Stack adapter | Provider author: native toolchain, build, test, packaging, run, diagnostics |
| Capability pack | Framework/domain owner: typed contracts and freedom slots, preferably derived from machine metadata |
| Organization assurance | Customer or assurance vendor: policies, validators, approvals, trust roots |

Synthesis-model, stack, and assurance providers are separate roles even when
one implementation supplies more than one role.

A provider must ship:

- versioned capability and effect contracts,
- native tool adapters and sandbox requirements,
- validators and evidence-producer declarations,
- compatibility and upgrade policy,
- conformance tests and provenance,
- cost and telemetry hooks.

A provider must never contain application-specific business logic or
benchmark-specific implementations. If a new claims, invoice, or demo
requirement needs a compiler or stack-provider code change, the architecture
has failed.

The first reference provider will be first-party and tied to a provider
compatibility corridor fixed before the first counted commercial interview.
The corridor declares language/runtime, supported framework-major range,
build/test interface, data boundary, and deployment shape. It must reflect a
real Brownfield market, not existing demo convenience, and remains immutable
through the provider and existence gates. No second target, registry,
marketplace, or broad capability catalog is funded before the existence gate
passes.

Provider economics are part of product economics. The measured cost includes
initial adapter and contract work, upgrades, conformance, support, and
amortization across changes. A plausible later business model is paid LTS
provider bundles, assurance integrations, and support; application-specific
business contracts remain customer-owned. This is a hypothesis to validate,
not a current claim.

Provider portability passes only if:

- a second real project uses the same provider with no compiler change and at
  most five person-days of project adapter work,
- a representative minor stack upgrade needs at most two person-days,
- an independent developer adds a non-trivial capability through the SDK and
  conformance suite without forking the compiler.

## 12. Reference Proof

The first vertical proof is one audit-intensive Brownfield business workflow
supplied by a qualified design partner. Provider portability is then proven on
a second independently supplied repository; the existence experiment expands
to the full four-or-more-repository clustered sample required by section 13.

The first system must include:

- persistent data and schema evolution,
- authentication, role authorization, and separation of duties,
- material business rules and state transitions,
- append-only audit history,
- external API or message boundary,
- automated static and executable checks,
- human approval policy,
- an emergency hotfix and reconciliation,
- at least ten sequential, interacting changes.

The provider and compiler may contain no scenario-specific implementation. The
native system must remain operable without SCULPT. A team not involved in the
compiler must perform at least one non-trivial change.

Existing Snake, showcase, CLI, GUI, and data-reconciliation examples are not
the reference proof.

## 13. Validation Protocol

### 13.1 Commercial Gate Before Product Engineering

Before implementing the new product architecture:

- freeze the provider compatibility corridor in section 11;
- conduct at most twenty qualified problem interviews within that corridor;
- obtain at least two independent external organizations willing to supply at
  least four real Brownfield repositories in that same corridor,
  representative change sequences, and review time,
- secure at least one paid pilot or equivalent binding commercial commitment.

If this gate fails, SCULPT stops. Interest in demos, GitHub stars, or praise for
the language idea does not substitute for the gate.

### 13.2 Architecture Gates

The minimal implementation must pass all of these before any comparative
benchmark:

1. **Language gate:** critical requirements are typed obligations; critical
   behavior does not depend on prose escape hatches or compiler-owned domain
   logic.
2. **Loop gate:** a real model repairs a real failing scenario through at least
   one diagnostic feedback iteration; no stub, replay, or hardcoded target
   output may satisfy it.
3. **Graph gate:** mutation tests demonstrate clean/incremental equivalence,
   critical invalidation recall, safe rename, semantic diff, and typed
   refinement. One unsound cache hit fails the gate.
4. **Operations gate:** the Brownfield unit and emergency-hotfix lifecycle work
   end to end without losing provenance.
5. **Provider gate:** the second real repository meets the portability and
   maintenance limits in section 11.

### 13.3 Three-Arm Existence Experiment

The decisive comparison has three arms:

1. **Full SCULPT:** typed graph, evidence, invalidation, and convergence loop.
2. **Ablation:** the same model and agent/test harness with the same structured
   specification, but no SCULPT typed graph or obligation engine.
3. **Best-practice baseline:** a current spec-driven coding agent operating
   natively with persistent instructions, policies, tests, hooks, CI, and
   lockfiles.

Arm 2 is essential. If it performs equally, the language and semantic kernel
are unnecessary.

Before holdout tasks exist, an independent baseline custodian runs a
preregistered bake-off of at least three eligible current agent/harness
configurations on non-holdout calibration tasks from the frozen provider
corridor. Every candidate receives the same operator qualification, setup and
tuning budget, tools, and time. The winner is selected by the preregistered
quality floor and fully loaded calibration cost, with deterministic tie-breaks.
Candidate set, operator, prompts, instructions, tuning changes, versions,
scores, and selection rule are published in the audit package. The selected
baseline is frozen before SCULPT holdout results are visible.

Before task selection, the protocol, providers, models, prompts, tool versions,
repositories, acceptance criteria, policies, metrics, exclusions, and analysis
scripts are frozen and content-addressed. All arms receive the same:

- model snapshot and inference settings,
- clean repository baseline and task statement,
- tool, network, secret, and compute permissions,
- visible tests and hidden acceptance tests,
- time, token, money, and human-intervention budgets,
- right to plan, build, test, diagnose, and repair.

The experiment uses at least four independent external Brownfield repositories
from at least two organizations in the frozen provider corridor. Each
repository contributes a frozen sequence of at least ten interacting changes,
for at least forty assigned changes in total.

Sequential changes within one repository are dependent repeated measures, not
independent samples. The repository-level sequence is the minimum inference
cluster. Arms are paired within the same clean repository and sequence, and the
preregistered analysis uses repository/sequence blocks with cluster-aware or
hierarchical inference. Power analysis is performed on that clustered design
and may require more repositories or sequences; it may never inflate power by
treating dependent changes as independent. A preregistered subset receives five
clean repetitions per arm for semantic reproducibility. Provider
implementation is frozen before holdout tasks are known.

Functional acceptance is scored by hidden tests. Independent reviewers assess
traceability and review effort without knowing which model produced a result;
where source format prevents full blinding, the limitation is recorded. Infra
failures, fallbacks, manual interventions, failed attempts, and exclusions are
reported rather than silently retried away.

All raw inputs, outputs, logs, hashes, and analysis code are retained for
adversarial review. For confidential partner repositories, the protocol
predeclares redaction and access rules: authorized independent auditors receive
the complete evidence, while public reporting contains reproducible method,
digests, non-confidential fixtures, and aggregate results. Confidentiality may
limit publication but may not hide a failed run or exclusion. Reproducibility
is measured against behavior and obligations, not only file hashes.

### 13.4 Primary Economic Metric

The primary metric is:

> fully loaded cost per policy-compliant accepted change over a sequential
> change series.

It includes:

- SCULPT or specification authoring and maintenance,
- provider, capability-pack, and contract setup amortization,
- model, judge, token, tool, build, test, and CI cost,
- failed candidates and repair loops,
- developer intervention,
- review, evidence assembly, audit, and approval time,
- hotfix and reconciliation effort,
- provider upgrades and ongoing maintenance.

The exact calculation is frozen before holdout selection:

~~~text
FLC10(arm) =
  all actual arm-attributable costs incurred through the first ten assigned
  changes of every repository sequence
  /
  number of policy-compliant accepted changes among those assigned changes
~~~

The numerator includes all one-time compiler/provider/contract/project setup,
even if expected to be reused later, plus every accepted, failed, abandoned, or
timed-out attempt. Shared experiment infrastructure is allocated equally by a
preregistered rule; arm-specific setup stays with that arm. Human time uses
role-specific fully loaded hourly rates in a fixed currency and valuation date,
declared before holdouts. External invoices and metered compute use actual
cost. A missing required cost invalidates the run. Zero accepted changes makes
FLC10 infinite. The change-ten gate is cumulative, not the marginal cost of the
tenth change. Learning curves after change ten are secondary only.

Secondary metrics include p50/p95 lead time, functional acceptance, escaped
defects, critical false-green rate, review/audit time, traceability accuracy,
invalidation recall and precision, cache savings, context consumption, and
cross-model portability. Source LOC is descriptive only.

### 13.5 Immutable Full-Go Gates

SCULPT receives Full-Go only if **all** conditions hold:

- functional acceptance is no more than five percentage points below the best
  baseline and no critical defect escapes;
- every critical obligation has a machine-traceable and independently correct
  mapping, mapping accuracy across all obligations is at least 95%, and
  critical false-green count is zero;
- FLC10 across the clustered repository sequences is at least 25% lower than
  the strongest baseline;
- blinded or limitation-adjusted review and audit time is at least 40% lower;
- Full SCULPT beats the no-graph ablation by at least 20% on the primary metric;
- critical invalidation recall is 100%, with no unsound cache hit;
- the provider portability gate passes on the second real repository;
- an external team completes a non-trivial governed change;
- at least two contributing external organizations remain willing to continue
  and at least one paid pilot has completed;
- an independent auditor confirms protocol conformance, baseline selection,
  cluster-aware analysis, cost calculation, and artifact provenance with no
  unresolved material finding.

Model cost may be higher only if the fully loaded primary metric still passes.
No single favorable showcase or secondary metric can compensate for a failed
gate.

### 13.6 Stop Rule

One preregistered remediation cycle is allowed only for identified
implementation defects or a correctable material audit finding, followed by an
exact rerun and reaudit of the failed gate.
Thresholds, tasks, exclusions, market, and baseline remain fixed.

If any gate still fails:

- stop SCULPT product development,
  - publish the negative result, method, and all non-confidential evidence, and
    provide the complete raw audit package to the authorized independent
    reviewers,
- archive the project,
- do not pivot back to more targets, demos, syntax work, or weaker benchmarks.

Passing all gates converts the bounded existence program into Full-Go. Only
then may the roadmap consider additional stacks, providers, or markets.

## 14. Current Reality And Immediate Reset

The current repository provides useful foundations:

- Rust CLI, parser, AST, diagnostics, and semantic checks,
- modules, scopes, contracts, and target separation,
- freeze/replay experiments,
- tests, examples, telemetry, and build infrastructure.

It does not yet implement the product described here:

- the current convergence path is retry-oriented rather than a verified
  compile-test-repair loop,
- no obligation/evidence system exists,
- no incremental typed semantic graph or evidence invalidation exists,
- current providers contain demo and domain-specific implementation logic,
- current benchmarks do not isolate the semantic-layer hypothesis.

Therefore:

- all existing benchmark verdicts are historical exploratory artifacts and
  provide no Go evidence for this concept;
- data-heavy deterministic-codegen passes are not evidence of model synthesis;
- prompt-first/vibe comparisons are retired as decision baselines;
- CLI, GUI, Web, TUI, Snake, showcase, theme, and broad contract work enter
  maintenance-only mode;
- no official benchmark runs before the architecture gates pass.

## 15. Execution Order

1. Quarantine historical benchmark claims and freeze the three existing targets
   to maintenance-only status.
2. Run the commercial gate and obtain the Brownfield repositories and change
   sequences.
3. Specify the minimal semantic graph, obligation/evidence records, closed-world
   permissions, ownership modes, and provider protocol.
4. Build one vertical unit: source -> graph -> bounded plan -> real synthesis ->
   failing verifier -> repair -> fresh evidence -> accepted native patch.
5. Add incremental invalidation, decision ledger, typed refinement, granular
   lock, emergency override, and reconciliation.
6. Prove the first provider on the second repository and through an independent
   SDK extension.
7. Freeze and run the three-arm existence experiment.
8. Submit artifacts to independent adversarial audit and apply the immutable
   Full-Go or stop decision.

No presentation or platform expansion work precedes these steps.

## 16. Risks The Concept Cannot Remove

The existence program must measure rather than explain away these risks:

- teams may reject a new language even with incremental adoption and an exit
  path;
- provider and contract maintenance may cost more than assurance saves;
- incumbent agent platforms may add typed semantics and evidence graphs;
- better models may erase the economic advantage;
- opaque native boundaries limit guarantees;
- tests, verifiers, policies, contracts, and reviewers can be wrong;
- auditors may not value SCULPT evidence artifacts;
- privacy and data-residency constraints may make model use uneconomic;
- debugging, hotfixes, and leaky abstractions may remain operationally painful;
- the team or market may be too small to sustain a compiler and provider.

No wording change resolves these risks. Only the gates do.

## 17. Ten-Year Relevance

SCULPT must not depend on models being bad at coding. Better models should make
candidate search cheaper and more capable.

The durable thesis is that consequential software changes still need:

- typed boundaries and ownership,
- explicit permissions and effects,
- persistent obligations,
- fresh and inspectable evidence,
- semantic impact analysis,
- controlled adoption and hotfix reconciliation,
- provider and model substitution,
- auditable team decisions.

If general coding platforms provide these properties with equal rigor and lower
cost, SCULPT should end rather than defend its syntax.

## 18. Canonical Pitch

> **SCULPT is the evidence-carrying change compiler for AI-written software.**
>
> Developers express the boundaries, invariants, freedoms, and proof required
> for a change as typed source. SCULPT lets untrusted compiler agents patch an
> existing system, then refuses acceptance until every affected critical
> obligation has fresh, policy-approved evidence.
>
> It is not a better prompt and not an app generator. It is a programmable
> assurance boundary between AI-generated change and production software.

## 19. Current Baseline References

The competitor definition in this document is grounded in current primary
documentation, not in a prompt-only caricature:

- [GitHub Spec Kit](https://github.github.com/spec-kit/) — structured
  specification-driven development, staged workflows, agent integrations, and
  extensibility.
- [GitHub Copilot customization](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-copilot-overview)
  — persistent repository context, custom agents, build, and test guidance.
- [Claude Code hooks](https://code.claude.com/docs/en/hooks-guide) —
  deterministic and agentic lifecycle gates around tool use and verification.
- [OpenAI Codex](https://openai.com/codex/) — end-to-end repository work,
  testing, skills, and multi-agent workflows.
