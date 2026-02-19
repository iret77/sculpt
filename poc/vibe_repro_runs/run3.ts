// This is a single-file TypeScript CLI for an Incident Triage Assistant.
// It has no external dependencies and can be run with ts-node or compiled with tsc.

// Required declarations to allow TypeScript to compile without @types/node
declare const process: any;
declare function require(name: string): any;

// Use the built-in Node.js readline module for interactive prompts
const readline = require("readline");

/**
 * Defines the structure for an incident triage plan.
 */
interface IncidentPlan {
  severity: 'CRITICAL' | 'HIGH' | 'MEDIUM';
  steps: [string, string, string, string]; // Exactly 4 actionable steps
  owner_hint: string;
  next_update_minutes: number;
}

/**
 * A map of incident choices to their corresponding triage plans.
 */
const INCIDENT_PLANS: Record<string, IncidentPlan> = {
  '1': {
    severity: 'CRITICAL',
    steps: [
      "Acknowledge the alert and declare a major incident.",
      "Engage the on-call engineer for the affected service via paging system.",
      "Post an initial update to the company-wide status page and internal channels.",
      "Attempt a service restart or a rollback of the last known stable deployment."
    ],
    owner_hint: 'Primary on-call for the affected service',
    next_update_minutes: 15,
  },
  '2': {
    severity: 'HIGH',
    steps: [
      "Acknowledge the alert and start an investigation.",
      "Analyze centralized logs and metrics dashboards to identify the source of errors.",
      "Check for recent deployments or configuration changes that correlate with the spike.",
      "Escalate to the owning team if the root cause is not immediately apparent within 30 minutes."
    ],
    owner_hint: 'Team owning the service with the error spike',
    next_update_minutes: 30,
  },
  '3': {
    severity: 'MEDIUM',
    steps: [
      "Acknowledge the alert and verify the latency increase with monitoring tools.",
      "Check database performance, query times, and resource utilization (CPU, memory).",
      "Inspect upstream/downstream service dependencies for bottlenecks.",
      "Consider scaling up resources if increased load is the confirmed cause."
    ],
    owner_hint: 'SRE or Infrastructure on-call team',
    next_update_minutes: 60,
  },
  '4': {
    severity: 'CRITICAL',
    steps: [
      "Immediately engage the security on-call team using the documented emergency protocol.",
      "Isolate the affected systems from the network to prevent lateral movement.",
      "Begin documenting a precise timeline of events and all actions taken.",
      "Restrict communication to a need-to-know basis to maintain confidentiality."
    ],
    owner_hint: 'Security Incident Response Team (SIRT)',
    next_update_minutes: 20,
  },
};

/**
 * Retrieves the incident plan for a given option and prints it as JSON.
 * @param option - The selected incident option ('1', '2', '3', or '4').
 */
function processAndPrintPlan(option: string): void {
  const plan = INCIDENT_PLANS[option];
  if (plan) {
    console.log(JSON.stringify(plan, null, 2));
  } else {
    console.error(`Error: Invalid option '${option}'. Please choose from 1, 2, 3, or 4.`);
    process.exit(1);
  }
}

/**
 * Runs the CLI in interactive mode, prompting the user for input.
 */
function runInteractiveMode(): void {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  console.log("Welcome to the Incident Triage Assistant.");
  console.log("Please select an incident type:");
  console.log("  1) Service Down");
  console.log("  2) Error Spike");
  console.log("  3) Latency Increase");
  console.log("  4) Security Incident");

  rl.question("\nEnter your choice (1-4): ", (answer: string) => {
    processAndPrintPlan(answer.trim());
    rl.close();
  });
}

/**
 * Runs the CLI in simulation mode based on command-line arguments.
 */
function runSimulateMode(): void {
  const simulateIndex = process.argv.indexOf('--simulate');
  
  if (simulateIndex === -1 || simulateIndex + 1 >= process.argv.length) {
    console.error("Error: --simulate flag requires an argument. Usage: --simulate <1|2|3|4>");
    process.exit(1);
    return;
  }
  
  const option = process.argv[simulateIndex + 1];
  processAndPrintPlan(option);
}

/**
 * Main function to determine the execution mode.
 */
function main(): void {
  const args = process.argv.slice(2); // Get arguments, excluding 'node' and script path
  if (args.includes('--simulate')) {
    runSimulateMode();
  } else {
    runInteractiveMode();
  }
}

// Execute the main function
main();