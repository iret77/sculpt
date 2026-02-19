// To run this file:
// 1. Save it as `triage.ts`
// 2. Compile it: `tsc triage.ts`
// 3. Run in interactive mode: `node triage.js`
// 4. Run in simulation mode: `node triage.js --simulate 2`

// These declarations allow the script to be written in TypeScript
// while referencing Node.js-specific globals without needing @types/node.
declare const process: any;
declare function require(name: string): any;

/**
 * Defines the structure for a single incident triage plan.
 */
interface TriagePlan {
    incident_type: string;
    severity: 'Critical' | 'High' | 'Medium';
    actionable_steps: [string, string, string, string]; // Ensures exactly 4 steps
    owner_hint: string;
    next_update_minutes: number;
}

/**
 * A record mapping user choices to their corresponding TriagePlan.
 */
const triagePlans: Record<string, TriagePlan> = {
    '1': {
        incident_type: 'Service Down',
        severity: 'Critical',
        actionable_steps: [
            "Declare a major incident and open a communication channel (e.g., Slack #incident-room).",
            "Check primary monitoring dashboards for the affected service (e.g., Grafana, Datadog).",
            "Page the on-call engineer for the service's primary team immediately.",
            "Verify customer impact through business metrics or customer support channels."
        ],
        owner_hint: 'SRE / On-call for the affected service',
        next_update_minutes: 15,
    },
    '2': {
        incident_type: 'Error Spike',
        severity: 'High',
        actionable_steps: [
            "Isolate the source of errors: a new deployment, specific user, or a particular region?",
            "Review recent code changes and deployments to the affected service.",
            "Analyze centralized logging (e.g., ELK, Splunk) for specific error messages and stack traces.",
            "Consider a controlled rollback of the most recent deployment if it is the likely cause."
        ],
        owner_hint: 'Development team responsible for the last deployment',
        next_update_minutes: 30,
    },
    '3': {
        incident_type: 'Latency Increase',
        severity: 'Medium',
        actionable_steps: [
            "Verify the latency increase across multiple monitoring tools and geographical regions.",
            "Investigate database query performance and external API call response times.",
            "Check for resource saturation on hosts (CPU, Memory, I/O, Network).",
            "Consult with the database administration or infrastructure team for deeper analysis."
        ],
        owner_hint: 'Infrastructure / Performance Engineering Team',
        next_update_minutes: 60,
    },
    '4': {
        incident_type: 'Security Incident',
        severity: 'Critical',
        actionable_steps: [
            "Immediately engage the Security Team via their dedicated emergency channel.",
            "Isolate the affected host(s) from the network to prevent potential lateral movement.",
            "Do NOT modify or delete anything on the affected system to preserve evidence.",
            "Begin documenting a precise timeline of events and observations in a secure location."
        ],
        owner_hint: 'Security Team / Incident Response Lead',
        next_update_minutes: 10,
    }
};

/**
 * Prints a TriagePlan to the console in a human-readable format.
 * @param plan The TriagePlan object to print.
 */
function printPlanForTerminal(plan: TriagePlan): void {
    console.log(`\n--- Triage Plan: ${plan.incident_type} ---`);
    console.log(`Severity: ${plan.severity}`);
    console.log(`Suggested Owner: ${plan.owner_hint}`);
    console.log(`Provide Next Update In: ${plan.next_update_minutes} minutes`);
    console.log('\nActionable Steps:');
    plan.actionable_steps.forEach((step, index) => {
        console.log(`  ${index + 1}. ${step}`);
    });
    console.log('----------------------------------------\n');
}

/**
 * Starts the interactive command-line interface to guide the user.
 */
function runInteractiveMode(): void {
    const readline = require("readline");
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
    });

    console.log('Welcome to the Incident Triage Assistant.');
    console.log('Please select the type of incident:');
    console.log('  1. Service Down');
    console.log('  2. Error Spike');
    console.log('  3. Latency Increase');
    console.log('  4. Security Incident');

    rl.question('\nEnter your choice (1-4): ', (answer: string) => {
        const plan = triagePlans[answer.trim()];
        if (plan) {
            printPlanForTerminal(plan);
        } else {
            console.error('\n[Error] Invalid choice. Please run the script again and select a number between 1 and 4.');
        }
        rl.close();
    });
}

/**
 * Handles the simulation mode by finding the correct plan and printing it as JSON.
 * @param choice The incident number (e.g., '1', '2') from the command line.
 */
function runSimulationMode(choice: string): void {
    const plan = triagePlans[choice];
    if (plan) {
        // Output the JSON string to stdout.
        console.log(JSON.stringify(plan, null, 2));
    } else {
        // Output a JSON error object to stderr and exit with an error code.
        const error = {
            error: `Invalid simulation choice '${choice}'. Must be 1, 2, 3, or 4.`
        };
        process.stderr.write(JSON.stringify(error, null, 2) + '\n');
        process.exit(1);
    }
}

/**
 * Main function to parse command-line arguments and determine the execution mode.
 */
function main(): void {
    const args: string[] = process.argv.slice(2);
    const simulateIndex = args.indexOf('--simulate');

    if (simulateIndex !== -1) {
        const choice = args[simulateIndex + 1];
        if (!choice) {
            const error = {
                error: 'The --simulate flag requires an argument (1, 2, 3, or 4).'
            };
            process.stderr.write(JSON.stringify(error, null, 2) + '\n');
            process.exit(1);
        }
        runSimulationMode(choice);
    } else {
        runInteractiveMode();
    }
}

// Start the application
main();