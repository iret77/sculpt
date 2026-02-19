// Add required declarations to satisfy TypeScript in a single-file context
// without a tsconfig.json or standard library types.
declare const process: any;
declare function require(name: string): any;

// Main entry point for the CLI tool.
(function main() {
    /**
     * Defines the structure for an incident triage plan.
     */
    interface TriagePlan {
        incident_type: string;
        severity: 'Critical' | 'High' | 'Medium' | 'Low';
        owner_hint: string;
        next_update_minutes: number;
        steps: [string, string, string, string]; // Ensures exactly 4 steps
    }

    /**
     * Data store for all available incident triage plans.
     * The key is the option number (string) for easy lookup.
     */
    const triagePlans = new Map<string, TriagePlan>([
        ['1', {
            incident_type: 'Service Down',
            severity: 'Critical',
            owner_hint: 'On-call SRE / Operations',
            next_update_minutes: 15,
            steps: [
                'Initiate a war room call with key stakeholders and the SRE team.',
                'Post an initial status update to the company-wide status page.',
                'Check primary monitoring dashboards for immediate anomalies (e.g., CPU, memory, network).',
                'Attempt a controlled service restart or trigger a rollback of the last deployment.',
            ],
        }],
        ['2', {
            incident_type: 'Error Spike',
            severity: 'High',
            owner_hint: 'Service Owning Team',
            next_update_minutes: 30,
            steps: [
                'Isolate the source of errors using distributed tracing and log aggregation tools.',
                'Analyze recent code changes or configuration updates for potential culprits.',
                'Check for external dependency failures (e.g., database, third-party APIs).',
                'Consider deploying a targeted hotfix or enabling a feature flag to mitigate the issue.',
            ],
        }],
        ['3', {
            incident_type: 'Latency Increase',
            severity: 'High',
            owner_hint: 'Performance Team / SRE',
            next_update_minutes: 30,
            steps: [
                'Review Application Performance Monitoring (APM) to identify slow transactions or queries.',
                'Investigate infrastructure metrics for resource contention (CPU, Memory, I/O, Network).',
                'Check database performance for long-running queries or connection pool exhaustion.',
                'Analyze traffic patterns for unexpected spikes or denial-of-service activity.',
            ],
        }],
        ['4', {
            incident_type: 'Security Incident',
            severity: 'Critical',
            owner_hint: 'Security / InfoSec Team',
            next_update_minutes: 10,
            steps: [
                'Immediately engage the security team and follow the established Security Incident Response Plan (SIRP).',
                'Isolate the affected systems from the network to contain the threat.',
                'Preserve evidence: take system snapshots, collect logs, and avoid making changes to compromised systems.',
                'Begin identifying the scope and impact of the breach; do NOT publicly disclose details without approval.',
            ],
        }],
    ]);

    /**
     * Prints the triage plan to the console in a human-readable format.
     * @param plan The TriagePlan object to print.
     */
    const printHumanReadablePlan = (plan: TriagePlan): void => {
        console.log(`\n--- Incident Triage Plan: ${plan.incident_type} ---`);
        console.log(`Severity: ${plan.severity}`);
        console.log(`Owner Hint: ${plan.owner_hint}`);
        console.log(`Next Update In: ${plan.next_update_minutes} minutes`);
        console.log('\nActionable Steps:');
        plan.steps.forEach((step, index) => {
            console.log(`${index + 1}. ${step}`);
        });
        console.log('-------------------------------------------\n');
    };

    /**
     * Prints the triage plan to the console as a JSON object.
     * @param plan The TriagePlan object to print.
     */
    const printJsonPlan = (plan: TriagePlan): void => {
        console.log(JSON.stringify(plan, null, 2));
    };

    /**
     * Handles the interactive mode, prompting the user for input.
     */
    const runInteractiveMode = (): void => {
        const readline = require("readline");
        const rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });

        console.log('Welcome to the Incident Triage Assistant.');
        console.log('Please select the type of incident:');
        triagePlans.forEach((plan, key) => {
            console.log(`  ${key}) ${plan.incident_type}`);
        });

        rl.question('\nEnter your choice (1-4): ', (answer: string) => {
            const plan = triagePlans.get(answer.trim());
            if (plan) {
                printHumanReadablePlan(plan);
            } else {
                console.error('\nError: Invalid choice. Please run the tool again and select a number between 1 and 4.');
            }
            rl.close();
        });
    };

    /**
     * Handles the simulation mode, printing JSON based on CLI arguments.
     * @param option The incident option number provided via CLI.
     */
    const runSimulationMode = (option: string): void => {
        const plan = triagePlans.get(option);
        if (plan) {
            printJsonPlan(plan);
        } else {
            console.error(`Error: Invalid simulation option '--simulate ${option}'. Please use a number from 1 to 4.`);
            process.exit(1);
        }
    };

    // --- Script Execution ---
    const args = process.argv.slice(2);
    const simulateIndex = args.indexOf('--simulate');

    if (simulateIndex !== -1 && args.length > simulateIndex + 1) {
        const option = args[simulateIndex + 1];
        runSimulationMode(option);
    } else {
        runInteractiveMode();
    }
})();