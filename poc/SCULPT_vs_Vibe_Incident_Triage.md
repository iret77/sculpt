# Case Study Run: SCULPT vs Vibe Coding (Incident Triage)

(C) 2026 byte5 GmbH

## 1) Experiment Scope

### Task
- Name: Incident Triage Assistant
- Description: Build a CLI tool that guides first-response actions for on-call incidents.
- Functional acceptance criteria:
  - [x] User can select incident class (`service down`, `error spike`, `latency increase`)
  - [x] Tool outputs concrete action plan per class
  - [x] Output is reliable across repeated runs

### Fixed Conditions
- LLM provider/model: `gemini / gemini-2.5-pro`
- Timebox per approach: `90 min`
- Same developer: `yes`
- Same machine/environment: `yes`
- Number of repeat runs per approach: `5`

## 2) Pre-Registered Expectations (Before Starting)
1. SCULPT will achieve higher reproducibility over repeated runs.
2. SCULPT will require fewer corrective iterations after change requests.
3. SCULPT will provide better control and clarity during refinement.

## 3) Approaches

### A) SCULPT
- Script path: `examples/incident_triage_assistant.sculpt`
- Build/run commands:
  - `sculpt build examples/incident_triage_assistant.sculpt --target cli --provider gemini`
  - `sculpt run examples/incident_triage_assistant.sculpt --target cli`

### B) Vibe Coding (Prompt-First)
- Prompt log path: `poc/vibe_prompts_incident_triage.md` (to be created during run)
- Generated code path: `poc/vibe_incident_triage_*` (to be captured during run)
- Build/run commands: `(captured during run)`

Actual artifacts:
- Prompt + iteration history: `poc/vibe_prompts_incident_triage.md`
- Iteration latency metrics: `poc/vibe_metrics.json`
- Reproducibility runs: `poc/vibe_repro_metrics.json`
- Final generated source: `poc/vibe_incident_triage.ts`

## 4) Hard Metrics

| Metric | SCULPT | Vibe | Winner | Notes |
|---|---:|---:|---|---|
| Time to first working version (min) | 0.48 | 2.62 | SCULPT | SCULPT first successful build in 28.78s. Vibe required 6 prompt iterations to compile cleanly. |
| Time to accepted final version (min) | 1.82 | 2.62 | SCULPT | SCULPT baseline + CR1 + CR2 build cycles. Vibe reached accepted code after iterative fixes. |
| Iterations until accepted | 3 | 6 | SCULPT | SCULPT: baseline + 2 change passes. Vibe: 6 LLM code iterations. |
| Reproducibility (stable runs / 5) | 5/5 | 0/5 | SCULPT | SCULPT produced identical target IR hash across 5 runs. Vibe produced 5 unique code hashes across 5 runs. |
| Regression count during changes | 0 | 3 | SCULPT | Vibe had multiple compile-breaking iterations before stabilization. |
| Change Request #1 effort (min) | 0.67 | 0.35 | Vibe | CR1 itself landed quickly in vibe iteration flow; SCULPT needed one full build cycle. |
| Change Request #2 effort (min) | 0.67 | 1.78 | SCULPT | Vibe needed multiple repair iterations after CR2. |
| Token/cost footprint (if available) | n/a | n/a | n/a | Token usage not exposed in current provider metrics. |

## 5) Developer UX Factors

| Factor | SCULPT | Vibe | Winner | Notes |
|---|---:|---:|---|---|
| Sense of control | 5 | 3 | SCULPT | SCULPT changes mapped directly to states/transitions; vibe required repair prompts. |
| Clarity of intent | 5 | 3 | SCULPT | Intent is explicit in SCULPT flow/state structure. |
| Flow continuity | 4 | 2 | SCULPT | Vibe loop was interrupted by compile-fix prompts. |
| Cognitive load | 4 | 3 | SCULPT | SCULPT required less prompt management overhead. |
| Refinement comfort | 4 | 2 | SCULPT | SCULPT edits were localized and predictable. |
| Trust in outputs | 5 | 2 | SCULPT | SCULPT output was stable across repeated runs; vibe varied each run. |
| Maintainability feel | 4 | 3 | SCULPT | SCULPT source stayed concise and structurally clear. |
| Team readability | 5 | 3 | SCULPT | SCULPT intent structure is easier to review than prompt+generated drift chain. |

## 6) Success Gate (Strict)
- [x] Reproducibility clearly higher than vibe coding
- [x] Change requests handled with fewer regressions and better stability
- [x] Developer UX score clearly better (not marginal)
- [x] Final quality at least equal

If one or more fail:
- [ ] Concrete improvement path identified
- [ ] If no credible path exists: mark No-Go

## 7) Post-Experiment Findings

### Observed Results vs Expectations
1. Expected:
   SCULPT reproducibility advantage.
   Observed: confirmed strongly (`5/5` stable hash vs `0/5` for vibe).
2. Expected:
   SCULPT fewer corrective iterations after changes.
   Observed: confirmed (SCULPT `3` total iterations vs vibe `6`, with vibe compile regressions).
3. Expected:
   Better control and clarity in refinement.
   Observed: confirmed in UX scoring (SCULPT leads across all 8 factors).

### Why SCULPT Won / Lost
- SCULPT won on determinism, consistency, and structured refinement.
- Vibe coding was competitive for quick change injection, but weaker on output stability.

### Concrete Learnings
1. SCULPT’s strongest differentiator in this case is reproducibility under repeated generation runs.
2. Prompt-first iteration can be fast locally, but repair loops increase unpredictability and review overhead.
3. Convergence controls plus structure are effective when the task is workflow-centric.

### Decision
- Outcome: `Go`
- Rationale: SCULPT shows a clear advantage over vibe coding on stability, control, and developer workflow quality for this task.
- Next actions:
  - Add token usage metrics to make cost comparison explicit.
  - Repeat this benchmark with one data-heavy task to test boundary conditions.
  - Expand case-study set and keep strict pre-registered gates.
