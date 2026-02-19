// These declarations are necessary for a single-file TypeScript CLI that interacts with Node.js globals.
declare const process: any;
declare function require(name: string): any;

/**
 * Defines the structure for an incident triage plan.
 */
interface TriagePlan {
  name: string;
  severity: 'Critical' | 'High' | 'Medium';
  steps: [string, string, string, string]; // A tuple type to enforce exactly 4 steps.
  owner_hint: string;
  next_update_minutes: number;
}

/**
 * A constant array holding the predefined triage plans for different incident types.
 */
const INCIDENT_PLANS: TriagePlan[] = [
  {
    name: "Service Down",
    severity: "Critical",
    steps: [
      "Confirm outage scope and impact with monitoring dashboards (e.g., Grafana, Datadog).",
      "Create a dedicated communication channel (e.g., Slack #incident-YYYY-MM-DD).",
      "Engage the on-call engineer for the affected service.",
      "Prepare to initiate a rollback of the last deployment if identified as a potential cause."
    ],
    owner_hint: "On-call SRE or Service Owner",
    next_update_minutes: 15
  },
  {
    name: "Error Spike",
    severity: "High",
    steps: [
      "Isolate the source of errors using logging platforms (e.g., Splunk, ELK).",
      "Identify the specific error type and its frequency.",
      "Check recent code changes or configuration updates for correlation.",
      "Consider enabling a feature flag to disable the problematic feature if possible."
    ],
    owner_hint: "Primary service development team",
    next_update_minutes: 30
  },
  {
    name: "Latency Increase",
    severity: "Medium",
    steps: [
      "Analyze application performance monitoring (APM) traces to find bottlenecks.",
      "Check database query performance and resource utilization (CPU, memory).",
      "Investigate upstream or downstream service dependencies for performance degradation.",
      "Review recent traffic pattern changes or spikes."
    ],
    owner_hint: "Performance engineering team or Service Owner",
    next_update_minutes: 60
  },
  {
    name: "Security Incident",
    severity: "Critical",
    steps: [
      "IMMEDIATELY engage the security team and follow the established Security Incident Response Plan (SIRP).",
      "Isolate the affected systems from the network to contain the threat.",
      "Preserve evidence: do not delete logs or reboot systems without security team approval.",
      "Begin documenting a timeline of events and actions taken in a secure location."
    ],
    owner_hint: "Security Incident Response Team (SIRT)",
    next_update_minutes: 10
  }
];

/**
 * Retrieves a triage plan by its 1-based index.
 * @param index - The 1-based index of the plan (1-4).
 * @returns The TriagePlan or undefined if the index is out of bounds.
 */
function getPlanByIndex(index: number): TriagePlan | undefined {
  return INCIDENT_PLANS[index - 1];
}

/**
 * Formats a TriagePlan for human-readable display in the console.
 * @param plan - The TriagePlan object to format.
 * @returns A formatted string ready for printing.
 */
function formatPlanForDisplay(plan: TriagePlan): string {
  const border = "=".repeat(60);
  const steps = plan.steps.map((step, i) => `  ${i + 1}. ${step}`).join('\n');

  return `
${border}
INCIDENT TRIAGE PLAN: ${plan.name}
${border}
Severity:          ${plan.severity}
Suggested Owner:   ${plan.owner_hint}
Next Update In:    ${plan.next_update_minutes} minutes

Actionable Steps:
${steps}
${border}
`;
}

/**
 * Handles the --simulate flag. It finds the corresponding plan,
 * prints it as a JSON string to stdout, and exits the process.
 * @param choice - The numeric choice (as a string) from the command line argument.
 */
function runSimulationMode(choice: string): void {
  const index = parseInt(choice, 10);
  const plan = getPlanByIndex(index);

  if (!plan) {
    console.error(`Error: Invalid simulation choice '${choice}'. Please use a number between 1 and ${INCIDENT_PLANS.length}.`);
    process.exit(1);
  }

  console.log(JSON.stringify(plan, null, 2));
  process.exit(0);
}

/**
 * Runs the interactive CLI prompt, asking the user to choose an incident
 * type and then displaying the corresponding triage plan.
 */
function runInteractiveMode(): void {
  const readline = require("readline");
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });

  console.log("Welcome to the Incident Triage Assistant.");
  console.log("Please select the type of incident:");

  INCIDENT_PLANS.forEach((plan, index) => {
    console.log(`  ${index + 1}. ${plan.name}`);
  });
  console.log("");

  rl.question(`Enter your choice (1-${INCIDENT_PLANS.length}): `, (answer: string) => {
    const index = parseInt(answer, 10);
    const plan = getPlanByIndex(index);

    if (plan) {
      console.log(formatPlanForDisplay(plan));
    } else {
      console.error(`\nInvalid choice '${answer}'. Please run the script again and select a valid number.`);
    }

    rl.close();
  });
}

/**
 * The main entry point of the script. It parses command-line arguments
 * to determine whether to run in simulation or interactive mode.
 */
function main(): void {
  const args = process.argv.slice(2);
  const simulateFlagIndex = args.indexOf('--simulate');

  if (simulateFlagIndex !== -1 && args[simulateFlagIndex + 1]) {
    const choice = args[simulateFlagIndex + 1];
    runSimulationMode(choice);
  } else {
    runInteractiveMode();
  }
}

// Execute the main function to start the application.
main();