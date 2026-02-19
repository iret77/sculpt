# Case Study Run: SCULPT vs Vibe Coding (Incident Triage)

(C) 2026 byte5 GmbH

## 1) Experiment Scope

### Task
- Name: Incident Triage Assistant
- Description: Build a CLI tool that guides first-response actions for on-call incidents.
- Functional acceptance criteria:
  - [ ] User can select incident class (`service down`, `error spike`, `latency increase`)
  - [ ] Tool outputs concrete action plan per class
  - [ ] Output is reliable across repeated runs

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

## 4) Hard Metrics

| Metric | SCULPT | Vibe | Winner | Notes |
|---|---:|---:|---|---|
| Time to first working version (min) |  |  |  |  |
| Time to accepted final version (min) |  |  |  |  |
| Iterations until accepted |  |  |  |  |
| Reproducibility (stable runs / 5) |  |  |  |  |
| Regression count during changes |  |  |  |  |
| Change Request #1 effort (min) |  |  |  |  |
| Change Request #2 effort (min) |  |  |  |  |
| Token/cost footprint (if available) |  |  |  |  |

## 5) Developer UX Factors

| Factor | SCULPT | Vibe | Winner | Notes |
|---|---:|---:|---|---|
| Sense of control |  |  |  |  |
| Clarity of intent |  |  |  |  |
| Flow continuity |  |  |  |  |
| Cognitive load |  |  |  |  |
| Refinement comfort |  |  |  |  |
| Trust in outputs |  |  |  |  |
| Maintainability feel |  |  |  |  |
| Team readability |  |  |  |  |

## 6) Success Gate (Strict)
- [ ] Reproducibility clearly higher than vibe coding
- [ ] Change requests handled with fewer regressions and better stability
- [ ] Developer UX score clearly better (not marginal)
- [ ] Final quality at least equal

If one or more fail:
- [ ] Concrete improvement path identified
- [ ] If no credible path exists: mark No-Go

## 7) Post-Experiment Findings

### Observed Results vs Expectations
1. Expected:
   Observed:
2. Expected:
   Observed:
3. Expected:
   Observed:

### Why SCULPT Won / Lost
- 

### Concrete Learnings
1.  
2.  
3.  

### Decision
- Outcome: `Go` / `Conditional Go` / `No-Go`
- Rationale:
- Next actions:

