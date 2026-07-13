# SCULPT Existence Experiment Template

Status: replaces the former SCULPT-versus-vibe template. The legacy filename is
retained to avoid broken links.

This template may be instantiated only after gates G0-G5 in the
[Open Work Register](../../docs/SCULPT_Open_Work_Register.md) pass.

## 1. Registration

| Field | Value |
|---|---|
| Protocol ID | |
| Registration timestamp | |
| Immutable protocol digest | |
| Analysis-code digest | |
| Holdout custodian | |
| Baseline custodian | |
| Independent reviewers | |
| Provider compatibility corridor digest | |
| Cluster-aware power analysis | |
| Independent inference clusters | At least 4 repository sequences |
| Assigned changes | At least 10 per repository, 40 total |
| External repositories/organizations | At least 4 repositories / 2 organizations |
| Reproducibility subset and repetitions | Five clean repetitions per arm |
| Allowed remediation cycle | One exact rerun/reaudit after implementation fix or correctable audit finding |

Protocol, thresholds, arms, tasks, exclusions, models, providers, tool versions,
and analysis must be frozen before holdout task selection.

## 2. Experiment Arms

| Arm | Required setup |
|---|---|
| Full SCULPT | Typed graph, closed-world permissions, obligation/evidence engine, invalidation, convergence loop |
| No-graph ablation | Same model, structured spec, agent/test harness, and tools; no SCULPT graph or obligation engine |
| Best-practice native baseline | Current spec-driven coding agent, persistent repo instructions, policies, tests, hooks, CI, locks |

## 3. Baseline Qualification

Before holdout tasks exist, an independent custodian evaluates at least three
eligible current agent/harness configurations on non-holdout calibration tasks
inside the frozen provider corridor.

| Candidate | Version/digest | Operator | Setup/tuning budget | Quality floor | Fully loaded cost | Result |
|---|---|---|---:|---:|---:|---|
| | | | | | | |

All candidates receive the same qualified operator, tools, time, setup, and
tuning budget. The preregistered quality-floor, cost, and deterministic
tie-break rule select the baseline. Candidate set, tuning changes, prompts,
instructions, results, and selection remain in the audit package. The winner is
frozen before any SCULPT holdout result is visible.

## 4. Equal Conditions

- exact model snapshot and inference settings;
- clean repository baseline;
- task statement and visible context;
- tools, filesystem, network, secrets, and compute permissions;
- visible tests and hidden acceptance tests;
- time, token, money, and human-intervention budgets;
- full plan-build-test-repair rights;
- frozen provider before holdout tasks are known.

Record every deviation. No silent fallback, retry filtering, or manual patch.

## 5. Clustered Change Sequences

| Cluster | Organization | Repository | Sequence digest | Change count | Critical obligations | Hidden tests |
|---:|---|---|---|---:|---|---|
| 1 | | | | 10 | | |

Changes inside a repository sequence are sequential, interacting, and
statistically dependent. The repository sequence is the minimum inference
cluster. Arms are paired from the same clean repository and sequence. The
preregistered model uses repository/sequence blocks and cluster-aware or
hierarchical inference. Power analysis may demand additional repositories or
sequences; it may not count dependent changes as independent samples.

## 6. Raw Capture

For every run retain:

- exact inputs and repository digest;
- SCULPT/spec source and changes;
- prompts, structured agent messages, tool calls, and diagnostics;
- candidate patches, failed attempts, and final patch;
- visible and hidden test results;
- obligation, decision, evidence, invalidation, and lock records;
- provider/model/tool versions and configuration;
- tokens, cost, latency, CI/build compute, and cache behavior;
- human interventions, review time, audit time, and approvals;
- infra failures, fallbacks, exclusions, and reasons.

Confidential repositories require preregistered redaction and access rules.
Authorized independent reviewers receive the complete raw package; public
reporting contains the method, digests, non-confidential fixtures, aggregate
results, and every failure/exclusion.

## 7. Primary Metric

> Fully loaded cost per policy-compliant accepted change over the sequential
> series.

Include modeling, provider/contract setup and amortization, model/judge/tool/CI
cost, failed attempts, developer intervention, review, evidence assembly,
audit, hotfix, reconciliation, upgrades, and maintenance.

Freeze the exact formula before holdout selection:

~~~text
FLC10(arm) =
  all actual arm-attributable costs through the first ten assigned changes
  of every repository sequence
  /
  policy-compliant accepted changes among those assigned changes
~~~

The numerator contains all one-time setup and every failed, abandoned, or
timed-out attempt. Shared infrastructure follows a frozen equal-allocation
rule; arm-specific setup remains arm-specific. Freeze role-specific fully
loaded hourly rates, currency, and valuation date. Use actual external and
metered costs. Missing cost data invalidates the run; zero accepted changes
makes FLC10 infinite. The gate is cumulative through change ten, not the
marginal tenth-change cost.

## 8. Secondary Metrics

- functional acceptance and escaped defects;
- critical false-green count;
- obligation/evidence traceability coverage and mapping accuracy;
- p50/p95 accepted-change lead time;
- independent review and audit time;
- invalidation recall and precision;
- clean/incremental equivalence;
- cache and context savings;
- semantic reproducibility;
- cross-model portability.

LOC and file hashes are descriptive only.

## 9. Immutable Full-Go Gate

- [ ] Functional acceptance no more than five percentage points below the best
      baseline.
- [ ] No escaped critical defect.
- [ ] Every critical obligation has a machine-traceable and independently
      correct evidence mapping.
- [ ] Overall independently checked evidence mapping accuracy at least 95%.
- [ ] Zero critical false-green.
- [ ] FLC10 across clustered repository sequences at least 25% below the
      strongest baseline.
- [ ] Review/audit time at least 40% below baseline.
- [ ] Full SCULPT at least 20% better than the no-graph ablation on the primary
      metric.
- [ ] Critical invalidation recall 100%; no unsound cache hit.
- [ ] Provider portability and independent-use gates pass.
- [ ] Commercial-continuation gate passes.
- [ ] Independent audit validates baseline selection, clustered inference,
      FLC10 calculation, protocol conformance, and provenance without an
      unresolved material finding.

All boxes must pass. A secondary win cannot compensate for failure.

## 10. Decision

| Result | Action |
|---|---|
| Experiment thresholds provisionally pass | Independent artifact audit; no Full-Go yet |
| Audit passes with no unresolved material finding | Full-Go |
| First failure caused by identified implementation defect | One preregistered fix and exact rerun |
| Any gate or reaudit still fails | Publish negative result and archive SCULPT |

The competitor, thresholds, tasks, exclusions, and target market remain fixed
during remediation.
