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
