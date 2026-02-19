// To run this file:
// 1. Save it as `triage.ts`
// 2. Install TypeScript and ts-node: `npm install -g typescript ts-node`
// 3. Run in interactive mode: `ts-node triage.ts`
// 4. Run in simulate mode: `ts-node triage.ts --simulate 2`

// Bypasses the need for @types/node by declaring global Node.js objects.
declare const process: any;
declare function require(name: string): any;

// Use the built-in Node.js readline module for interactive prompts.
const readline = require("readline");

/**
 * Defines the structure for a triage plan for a specific incident type.
 */
interface TriagePlan {
  name: string;
  severity: "critical" | "high" | "medium";
  actionable_steps: [string, string, string, string]; // Tuple to enforce exactly 4 steps.
  owner_hint: string;
  next_update_minutes: number;
}

/**
 * A map of incident options to their corresponding triage plans.
 */
const TRIAGE_PLANS: { [key: number]: TriagePlan } = {
  1: {
    name: "Service Down",
    severity: "critical",
    actionable_steps: [
      "Acknowledge the alert and declare a major incident in the communication channel (e.g., Slack #incidents).",
      "Assemble the core incident response team by paging the primary on-call for related services.",
      "Review primary service dashboards and recent deployment pipelines to identify an immediate cause.",
      "Establish a communication cadence and prepare the first internal status update.",
    ],
    owner_hint: "On-call SRE/DevOps",
    next_update_minutes: 5,
  },
  2: {
    name: "Error Spike",
    severity: "high",
    actionable_steps: [
      "Analyze aggregated logs (e.g., Splunk, ELK) for the specific error signature and frequency.",
      "Correlate the spike's start time with recent code deployments or feature flag changes.",
      "Isolate the impact: Is it affecting all users, a specific region, or a subset of customers?",
      "If a recent change is identified as the likely cause, prepare and execute a targeted rollback plan.",
    ],
    owner_hint: "Primary service development team",
    next_update_minutes: 10,
  },
  3: {
    name: "Latency Increase",
    severity: "medium",
    actionable_steps: [
      "Check Application Performance Monitoring (APM) tools for slow transactions or database queries.",
      "Inspect infrastructure metrics for resource saturation (CPU, Memory, I/O) on relevant hosts.",
      "Review network dashboards for increased traffic, packet loss, or dependency latency.",
      "Notify dependent teams if upstream or downstream services are identified as the source of latency.",
    ],
    owner_hint: "On-call SRE or relevant performance-focused dev team",
    next_update_minutes: 15,
  },
  4: {
    name: "Security Incident",
    severity: "critical",
    actionable_steps: [
      "Immediately engage the security on-call team following the documented 'break-glass' procedure.",
      "Isolate the affected system(s) from the network to prevent lateral movement while preserving its state.",
      "Begin evidence preservation by taking memory dumps and disk snapshots. Do not modify or delete files.",
      "Restrict all communication to a secure, private channel and follow the security team's lead on all actions.",
    ],
    owner_hint: "Security Incident Response Team (SIRT)",
    next_update_minutes: 30,
  },
};

/**
 * Retrieves the triage plan for a given option and formats it for output.
 * @param option The number corresponding to the incident type.
 * @returns A JSON string of the triage plan or null if the option is invalid.
 */
function getTriagePlanAsJson(option: number): string | null {
  const plan = TRIAGE_PLANS[option];
  if (!plan) {
    return null;
  }
  
  const result = {
    severity: plan.severity,
    actionable_steps: plan.actionable_steps,
    owner_hint: plan.owner_hint,
    next_update_minutes: plan.next_update_minutes,
  };

  return JSON.stringify(result, null, 2);
}

/**
 * Displays the main menu for interactive mode.
 */
function showMenu(): void {
  console.log("\n--- Incident Triage Assistant ---");
  console.log("Please select the type of incident:");
  for (const key in TRIAGE_PLANS) {
    console.log(`  ${key}: ${TRIAGE_PLANS[key].name}`);
  }
  console.log("---------------------------------");
}

/**
 * Handles the interactive mode of the CLI.
 */
function runInteractiveMode(): void {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  showMenu();

  rl.question("Enter your choice (1-4): ", (answer: string) => {
    const option = parseInt(answer, 10);
    const resultJson = getTriagePlanAsJson(option);

    if (resultJson) {
      console.log(resultJson);
    } else {
      console.error("\nInvalid input. Please enter a number between 1 and 4.");
    }
    rl.close();
  });
}

/**
 * Handles the simulation mode of the CLI.
 * @param args The command line arguments.
 */
function runSimulateMode(args: string[]): void {
  const simulateIndex = args.indexOf("--simulate");
  const optionArg = args[simulateIndex + 1];

  if (!optionArg) {
    console.error("Error: --simulate flag requires a number (1-4).");
    process.exit(1);
  }

  const option = parseInt(optionArg, 10);
  const resultJson = getTriagePlanAsJson(option);

  if (resultJson) {
    console.log(resultJson);
    process.exit(0);
  } else {
    console.error(`Error: Invalid option '${optionArg}'. Please use 1, 2, 3, or 4.`);
    process.exit(1);
  }
}

/**
 * Main function to start the CLI application.
 * It checks for the `--simulate` flag to determine the execution mode.
 */
(function main() {
  const args = process.argv.slice(2);
  if (args.includes("--simulate")) {
    runSimulateMode(args);
  } else {
    runInteractiveMode();
  }
})();