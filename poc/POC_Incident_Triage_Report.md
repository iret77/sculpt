# Case Study: Incident Triage Assistant (SCULPT vs Classical Code)

(C) 2026 byte5 GmbH

## Scope
This case study compares two implementations of the same real-world task:
an incident triage assistant that provides first-response action plans for on-call engineers.

Implementations:
- SCULPT: `examples/incident_triage_assistant.sculpt`
- Classical TypeScript: `poc/classic_incident_triage.ts`

## Compared Source Code

### SCULPT Source (`examples/incident_triage_assistant.sculpt`)
```sculpt
# Incident Triage Assistant (PoC)
# Real-world task: guide on-call engineers to a first-response action plan.

@meta target=cli
@meta nd_budget=30
@meta confidence=0.85

module(Ops.Incident.Triage)

  flow(Main)
    start > Intro

    state(Intro)
      render text("INCIDENT TRIAGE ASSISTANT", color: "yellow")
      render text("Pick incident type:", color: "blue")
      render text("1 = Service down", color: "white")
      render text("2 = Error spike", color: "white")
      render text("3 = Latency increase", color: "white")
      render text("Esc = Exit", color: "white")
      on key(1) > ServiceDown
      on key(2) > ErrorSpike
      on key(3) > Latency
      on key(esc) > Exit
    end

    state(ServiceDown)
      render text("SERVICE DOWN", color: "red")
      render text("Action plan:", color: "yellow")
      render text("- Declare SEV-1", color: "white")
      render text("- Start status page incident", color: "white")
      render text("- Assign commander + comms owner", color: "white")
      render text("- Roll back latest deploy if recent", color: "white")
      render text("Enter = Back", color: "blue")
      on key(enter) > Intro
    end

    state(ErrorSpike)
      render text("ERROR SPIKE", color: "magenta")
      render text("Action plan:", color: "yellow")
      render text("- Check top failing endpoint", color: "white")
      render text("- Compare release/version deltas", color: "white")
      render text("- Enable degraded mode if available", color: "white")
      render text("- Page owning team if >5 min sustained", color: "white")
      render text("Enter = Back", color: "blue")
      on key(enter) > Intro
    end

    state(Latency)
      render text("LATENCY INCREASE", color: "cyan")
      render text("Action plan:", color: "yellow")
      render text("- Check DB and cache saturation", color: "white")
      render text("- Inspect queue backlog", color: "white")
      render text("- Apply temporary rate-limit if needed", color: "white")
      render text("- Capture flamegraph before restart", color: "white")
      render text("Enter = Back", color: "blue")
      on key(enter) > Intro
    end

    state(Exit)
      render text("Session closed. Stay calm and log your actions.", color: "green")
      terminate
    end
  end

  # Constrain wording and structure for lower ND.
  nd(incidentPlaybookShape)
    propose responseGuide(format: "step-list", audience: "on-call")
    satisfy(
      hasClearTitle(),
      hasActionableSteps(min: 4),
      usesOperationalLanguage(),
      supportsQuickKeyNavigation()
    )
  end

end
```

### Classical Source (`poc/classic_incident_triage.ts`)
```ts
#!/usr/bin/env node

declare const process: any;

type IncidentKey = "1" | "2" | "3";

type Playbook = {
  title: string;
  severity: "SEV-1" | "SEV-2";
  steps: string[];
};

const PLAYBOOKS: Record<IncidentKey, Playbook> = {
  "1": {
    title: "Service Down",
    severity: "SEV-1",
    steps: [
      "Declare SEV-1 and open incident channel.",
      "Start status page incident update.",
      "Assign incident commander and comms owner.",
      "Roll back latest deploy if change is recent.",
    ],
  },
  "2": {
    title: "Error Spike",
    severity: "SEV-2",
    steps: [
      "Identify top failing endpoint and error class.",
      "Compare release and config delta from baseline.",
      "Enable degraded mode or feature flag fallback.",
      "Page owning team if sustained longer than 5 minutes.",
    ],
  },
  "3": {
    title: "Latency Increase",
    severity: "SEV-2",
    steps: [
      "Check DB, cache, and dependency saturation.",
      "Inspect queue backlog and worker health.",
      "Apply temporary rate limit if saturation continues.",
      "Capture flamegraph/profile before restart actions.",
    ],
  },
};

function parseSimulateArg(argv: string[]): IncidentKey | null {
  const idx = argv.findIndex((x) => x === "--simulate");
  if (idx === -1) return null;
  const value = argv[idx + 1];
  if (value === "1" || value === "2" || value === "3") return value;
  console.error("Usage: --simulate <1|2|3>");
  process.exit(2);
}

function buildResult(choice: IncidentKey) {
  const book = PLAYBOOKS[choice];
  return {
    incident_type: book.title,
    severity: book.severity,
    recommended_actions: book.steps,
    generated_at_utc: new Date().toISOString(),
  };
}

function main() {
  const sim = parseSimulateArg(process.argv.slice(2));
  if (!sim) {
    console.log("Use --simulate <1|2|3> for this PoC run.");
    process.exit(0);
  }
  console.log(JSON.stringify(buildResult(sim), null, 2));
}

main();
```

## Task Requirements
Both implementations must:
- present incident categories,
- return a concrete action plan,
- run locally with correct output.

## Build And Run Summary

### SCULPT implementation
- Build: `sculpt build examples/incident_triage_assistant.sculpt --target cli --provider gemini`
- Artifacts generated in `dist/incident_triage_assistant/`:
  - `ir.json`
  - `target.ir.json`
  - `nondet.report`
  - `build.meta.json`
- Flow transitions and state output were generated and validated.

### Classical implementation
- Build: `npx -y -p typescript tsc poc/classic_incident_triage.ts --target ES2020 --module commonjs --outDir dist/poc`
- Run: `node dist/poc/classic_incident_triage.js --simulate 1`
- Output includes incident type, severity, and action steps in deterministic JSON.

## Results
Both solutions met the functional requirements:
- valid incident categories,
- actionable response plans,
- successful local execution.

## Comparative Findings

### Where SCULPT performed well
- Fast expression of interaction flow (`Intro -> Incident State -> Back/Exit`).
- Clear separation of intent, constraints, and target generation.
- Explicit convergence metadata (`nd_budget`, `confidence`) and ND reporting.

### Where classical TypeScript performed well
- Direct deterministic control over behavior and output structure.
- Straightforward scripting/test automation patterns.
- No dependency on LLM output quality for core logic correctness.

### Current SCULPT limitations observed in this case
- Build latency depends on LLM compile step.
- Output quality still depends on provider/model behavior.
- CLI target currently favors flow-style interaction over richer data processing patterns.

## Developer Workflow Assessment (Neutral)

### SCULPT workflow characteristics
- Strong for flow-centric and intent-centric problem framing.
- Good for rapid structure-first iterations.
- Requires clear constraints to avoid variability.

### Classical workflow characteristics
- Strong for implementation-detail control and predictable behavior.
- Better fit for low-level logic and strict data transformations.
- Requires more manual coding effort for workflow orchestration.

## Conclusion
This case study indicates that SCULPT already provides practical value for workflow-oriented operational tools.
It is currently not a full replacement for classical languages in deterministic, implementation-heavy domains.

The realistic near-term positioning is:
- SCULPT for intent/flow-centric software with controlled convergence.
- Classical languages for low-level deterministic logic and heavy data mechanics.

## Key Learnings
1. Convergence controls (`nd_budget`, `confidence`) improve practical governance.
2. ND reporting is useful and should remain first-class.
3. Reproducibility (`freeze/replay`) remains critical for team-scale adoption.
4. To compete with AI-assisted coding baselines, SCULPT must continue reducing latency and expanding deterministic guardrails.
