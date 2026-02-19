#!/usr/bin/env node

// Incident Triage Assistant
// A single-file TypeScript CLI for providing initial incident response steps.
//
// To Run:
// 1. No external dependencies are needed.
// 2. Execute with ts-node: ts-node ./incident-triage.ts
//
// Usage:
//   Interactive Mode: ./incident-triage.ts
//   Automation/JSON Mode: ./incident-triage.ts --simulate <1|2|3|4>

import * as readline from 'readline';

// --- Type Declarations ---

// Avoids the need for @types/node by declaring the global 'process' object.
declare const process: any;

// --- Data Model ---

interface IncidentPlan {
  id: number;
  name: string;
  description: string;
  severity: 'Critical' | 'High' | 'Medium' | 'SEV-1';
  steps: string[];
  owner_hint: string;
  next_update_minutes: number;
}

const INCIDENT_PLANS: readonly IncidentPlan[] = [
  {
    id: 1,
    name: 'Service is down',
    description: 'The entire service or a critical endpoint is unresponsive.',
    severity: 'Critical',
    steps: [
      'ACKNOWLEDGE & ESCALATE: Page the primary on-call engineer and the designated incident commander immediately.',
      'COMMUNICATE: Post an initial update to the internal status page and relevant team channels (e.g., #incidents).',
      'INVESTIGATE: Check core infrastructure dashboards (CPU, Memory, Network) and recent deployment pipelines for failures.',
      'MITIGATE: If a recent deployment is identified as the likely cause, initiate the rollback procedure without delay.',
    ],
    owner_hint: 'Site Reliability Engineering (SRE) / On-call',
    next_update_minutes: 15,
  },
  {
    id: 2,
    name: 'Error rate has spiked',
    description: 'A significant increase in application errors (e.g., 5xx HTTP codes).',
    severity: 'High',
    steps: [
      'INVESTIGATE: Check error tracking software (e.g., Sentry, DataDog) for the specific error signature and frequency.',
      'ISOLATE: Determine if the spike is isolated to a specific service, endpoint, customer segment, or region.',
      'CORRELATE: Look for correlations with recent code deployments, feature flag changes, or infrastructure events.',
      'REMEDIATE: Develop and deploy a hotfix or roll back the offending change. If not possible, disable the feature via a flag.',
    ],
    owner_hint: 'Backend Service Team / On-call',
    next_update_minutes: 30,
  },
  {
    id: 3,
    name: 'Latency has increased',
    description: 'Response times are significantly higher than baseline.',
    severity: 'Medium',
    steps: [
      'INVESTIGATE: Analyze Application Performance Monitoring (APM) traces to identify slow transactions or database queries.',
      'CHECK DEPENDENCIES: Verify the health and response times of downstream services, databases, and third-party APIs.',
      'CHECK RESOURCES: Look for resource saturation (CPU, I/O, connection pools) on servers and databases.',
      'COMMUNICATE: Inform stakeholders about the performance degradation and post updates on the investigation.',
    ],
    owner_hint: 'Infrastructure Team or relevant Backend Team',
    next_update_minutes: 60,
  },
  {
    id: 4,
    name: 'Security Incident',
    description: 'Potential data breach, unauthorized access, or vulnerability exploit.',
    severity: 'SEV-1',
    steps: [
      'CONTAIN & ISOLATE: Immediately isolate the affected systems from the network to prevent lateral movement.',
      'ESCALATE: Page the on-call Security Team and Legal department immediately. Do not discuss details in public channels.',
      'PRESERVE EVIDENCE: Do not reboot or modify the affected systems. Create forensic snapshots or disk images if possible.',
      'COMMUNICATE CAREFULLY: Use secure, designated channels for all communications. Prepare for notifications as directed by the Security Team.',
    ],
    owner_hint: 'Security Team / CISO',
    next_update_minutes: 10,
  },
];

// --- Helper Functions ---

/**
 * Prints a formatted incident plan to the console for interactive mode.
 */
function displayPlan(plan: IncidentPlan): void {
  console.log('\n-----------------------------------------');
  console.log(`Incident Type: ${plan.name}`);
  console.log(`Severity: ${plan.severity}`);
  console.log(`Suggested Owner: ${plan.owner_hint}`);
  console.log(`Next Update Due: in ${plan.next_update_minutes} minutes`);
  console.log('-----------------------------------------\n');
  console.log('Recommended Initial Response Steps:');
  plan.steps.forEach((step, index) => {
    console.log(`  ${index + 1}. ${step}`);
  });
  console.log('\n');
}

/**
 * Handles the non-interactive simulation mode.
 * @param planId The ID of the plan to simulate.
 */
function runSimulation(planId: number): void {
  const plan = INCIDENT_PLANS.find((p) => p.id === planId);

  if (!plan) {
    const validIds = INCIDENT_PLANS.map(p => p.id).join(', ');
    console.error(`Error: Invalid plan ID "${planId}". Please use one of: ${validIds}.`);
    process.exit(1);
  }

  // Output the plan as a JSON object to stdout
  console.log(JSON.stringify(plan, null, 2));
  process.exit(0);
}

/**
 * Handles the interactive CLI menu mode using Node's built-in readline module.
 */
async function runInteractive(): Promise<void> {
  console.log('What type of incident are you seeing?');
  INCIDENT_PLANS.forEach((plan) => {
    console.log(`  ${plan.id}. ${plan.name} (${plan.description})`);
  });
  console.log('');

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  try {
    const answer = await new Promise<string>(resolve => {
        rl.question('Enter the number of the incident: ', resolve);
    });

    const planId = parseInt(answer.trim(), 10);

    if (isNaN(planId)) {
      console.error(`\nError: Invalid input. "${answer.trim()}" is not a number.`);
      process.exit(1);
    }

    const selectedPlan = INCIDENT_PLANS.find((p) => p.id === planId);

    if (selectedPlan) {
      displayPlan(selectedPlan);
    } else {
      console.error(`\nError: Invalid selection. No plan found with ID "${planId}".`);
      process.exit(1);
    }
  } catch (error) {
    console.error('\nThe interactive prompt failed to run.', error);
    process.exit(1);
  } finally {
    rl.close();
  }
}


// --- Main Execution Logic ---

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const simulateIndex = args.indexOf('--simulate');

  if (simulateIndex !== -1) {
    const planIdArg = args[simulateIndex + 1];
    if (!planIdArg) {
      const validIds = INCIDENT_PLANS.map(p => p.id).join(', ');
      console.error(`Error: The --simulate flag requires an argument (${validIds}).`);
      process.exit(1);
    }

    const planId = parseInt(planIdArg, 10);
    if (isNaN(planId)) {
      console.error(`Error: Invalid argument "${planIdArg}" for --simulate. It must be a number.`);
      process.exit(1);
    }

    runSimulation(planId);
  } else {
    await runInteractive();
  }
}

// Run the main function and catch any top-level errors.
main().catch((error) => {
  console.error('An unhandled error occurred:', error);
  process.exit(1);
});