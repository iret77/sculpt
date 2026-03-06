use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcCommand, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use glob::glob;
use sha2::{Digest, Sha256};

use crate::ai::{generate_target_ir, AiProvider, DebugCapture, TargetSpec};
use crate::build_meta::{dist_dir_for_input, now_unix_ms, write_build_meta, BuildMeta, TokenUsage};
use crate::contracts::{parse_target_contract, validate_module_against_contract};
use crate::convergence::{ConvergenceControls, FallbackMode};
use crate::freeze::{create_lock, read_lock, verify_lock, write_lock};
use crate::ir::{from_ast, to_pretty_json, IrModule};
use crate::parser::parse_source;
use crate::report::generate_report;
use crate::semantics::{format_diagnostics, has_errors, validate_module_with_imports};
use crate::target_ir::{from_json_value, TargetIr};
use crate::targets::{
    describe_target, emit_cli, emit_gui, emit_web, list_targets, resolve_target, run_cli,
    run_external_target, run_gui, run_web, TargetKind,
};
use crate::versioning::LANGUAGE_DEFAULT;
use serde_json::Value;

#[derive(Parser)]
#[command(
    name = "sculpt",
    version,
    about = "SCULPT compiler — (C) 2026 byte5 GmbH\nLanguage default: 1.0",
    after_help = "TUI: run `sculpt` with no arguments"
)]
pub struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Examples,
    Project {
        #[command(subcommand)]
        cmd: ProjectCommand,
    },
    Gate {
        #[command(subcommand)]
        cmd: GateCommand,
    },
    Benchmark {
        #[command(subcommand)]
        cmd: BenchmarkCommand,
    },
    Auth {
        #[command(subcommand)]
        cmd: AuthCommand,
    },
    Target {
        #[command(subcommand)]
        cmd: TargetCommand,
    },
    Build {
        input: PathBuf,
        #[arg(long)]
        target: Option<String>,
        #[arg(long = "nd-policy", value_parser = ["strict"])]
        nd_policy: Option<String>,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long, help = "Override model (defaults to provider config)")]
        model: Option<String>,
        #[arg(long)]
        strict_provider: bool,
        #[arg(long, value_name = "level", num_args = 0..=1, default_missing_value = "compact", value_parser = ["compact", "raw", "all", "json"])]
        debug: Option<String>,
    },
    Freeze {
        input: PathBuf,
        #[arg(long = "nd-policy", value_parser = ["strict"])]
        nd_policy: Option<String>,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long, help = "Override model (defaults to provider config)")]
        model: Option<String>,
        #[arg(long)]
        strict_provider: bool,
        #[arg(long)]
        target: Option<String>,
        #[arg(long, value_name = "level", num_args = 0..=1, default_missing_value = "compact", value_parser = ["compact", "raw", "all", "json"])]
        debug: Option<String>,
    },
    Replay {
        input: PathBuf,
        #[arg(long)]
        target: Option<String>,
    },
    Run {
        input: PathBuf,
        #[arg(long)]
        target: Option<String>,
    },
    Clean {
        input: Option<PathBuf>,
        #[arg(long)]
        all: bool,
        #[arg(long, help = "Remove entries older than N days")]
        max_age_days: Option<u64>,
        #[arg(long, help = "Keep only the newest N dist entries")]
        keep_latest: Option<usize>,
        #[arg(long, help = "Enforce max dist size in MB by deleting oldest entries")]
        max_size_mb: Option<u64>,
    },
}

#[derive(Subcommand)]
pub enum TargetCommand {
    List,
    Describe {
        #[arg(long)]
        target: String,
    },
    Packages {
        #[arg(long)]
        target: String,
    },
    Exports {
        #[arg(long)]
        target: String,
        #[arg(long)]
        package: String,
    },
    Stacks {
        #[arg(long)]
        target: String,
    },
}

#[derive(Subcommand)]
pub enum GateCommand {
    Check { gate_file: PathBuf },
}

#[derive(Subcommand)]
pub enum BenchmarkCommand {
    DataHeavy {
        #[arg(
            long,
            default_value = "examples/business/invoice_reconciliation_batch.sculpt"
        )]
        script: PathBuf,
        #[arg(long, default_value = "poc/data")]
        dataset_root: PathBuf,
        #[arg(long, default_value = "small,medium,large")]
        sizes: String,
        #[arg(long, default_value_t = 5)]
        repro_runs: usize,
        #[arg(long, default_value = "poc/data_heavy_sculpt_metrics.json")]
        output: PathBuf,
        #[arg(long, default_value = "poc/gates/data_heavy_sculpt_gate_input.json")]
        gate_output: PathBuf,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        strict_provider: bool,
        #[arg(long, default_value = "cli")]
        target: String,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommand {
    Create {
        name: String,
        #[arg(short = 'p', long = "path")]
        path: Option<PathBuf>,
        #[arg(short = 'f', long = "files", num_args = 1..)]
        files: Option<Vec<String>>,
    },
}

#[derive(Subcommand)]
pub enum AuthCommand {
    Check {
        #[arg(long, default_value = "openai")]
        provider: String,
        #[arg(long)]
        verify: bool,
    },
}

#[derive(Default, serde::Deserialize)]
struct Config {
    provider: Option<String>,
    openai: Option<OpenAIConfig>,
    anthropic: Option<AnthropicConfig>,
    gemini: Option<GeminiConfig>,
    clean: Option<CleanConfig>,
}

#[derive(Default, serde::Deserialize)]
struct OpenAIConfig {
    api_key: Option<String>,
    model: Option<String>,
}

#[derive(Default, serde::Deserialize)]
struct AnthropicConfig {
    api_key: Option<String>,
    model: Option<String>,
}

#[derive(Default, serde::Deserialize)]
struct GeminiConfig {
    api_key: Option<String>,
    model: Option<String>,
}

#[derive(Default, serde::Deserialize)]
struct CleanConfig {
    auto: Option<bool>,
    max_age_days: Option<u64>,
    keep_latest: Option<usize>,
    max_size_mb: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SculptProjectFile {
    name: Option<String>,
    entry: Option<String>,
    modules: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProjectContext {
    entry_module: String,
    modules: HashMap<String, (PathBuf, crate::ast::Module)>,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Examples => write_examples(),
        Command::Project { cmd } => match cmd {
            ProjectCommand::Create { name, path, files } => {
                project_create(&name, path.as_deref(), files.as_deref())
            }
        },
        Command::Gate { cmd } => match cmd {
            GateCommand::Check { gate_file } => gate_check(&gate_file),
        },
        Command::Benchmark { cmd } => match cmd {
            BenchmarkCommand::DataHeavy {
                script,
                dataset_root,
                sizes,
                repro_runs,
                output,
                gate_output,
                provider,
                model,
                strict_provider,
                target,
            } => benchmark_data_heavy(
                &script,
                &dataset_root,
                &sizes,
                repro_runs,
                &output,
                &gate_output,
                provider,
                model,
                strict_provider,
                &target,
            ),
        },
        Command::Auth { cmd } => match cmd {
            AuthCommand::Check { provider, verify } => auth_check(&provider, verify),
        },
        Command::Target { cmd } => match cmd {
            TargetCommand::List => target_list(),
            TargetCommand::Describe { target } => target_describe(&target),
            TargetCommand::Packages { target } => target_packages(&target),
            TargetCommand::Exports { target, package } => target_exports(&target, &package),
            TargetCommand::Stacks { target } => target_stacks(&target),
        },
        Command::Build {
            input,
            target,
            nd_policy,
            provider,
            model,
            strict_provider,
            debug,
        } => build(
            &input,
            target.as_deref(),
            nd_policy,
            provider,
            model,
            strict_provider,
            debug,
        ),
        Command::Freeze {
            input,
            nd_policy,
            provider,
            model,
            strict_provider,
            target,
            debug,
        } => freeze(
            &input,
            nd_policy,
            provider,
            model,
            strict_provider,
            target.as_deref(),
            debug,
        ),
        Command::Replay { input, target } => replay(&input, target.as_deref()),
        Command::Run { input, target } => run_cmd(&input, target.as_deref()),
        Command::Clean {
            input,
            all,
            max_age_days,
            keep_latest,
            max_size_mb,
        } => clean_cmd(
            input.as_deref(),
            all,
            CleanRetention {
                max_age_days,
                keep_latest,
                max_size_mb,
            },
        ),
    }
}

#[derive(Debug, serde::Deserialize)]
struct GateSpec {
    name: String,
    study: Option<String>,
    criteria: Vec<GateCriterion>,
}

#[derive(Debug, serde::Deserialize)]
struct GateCriterion {
    id: String,
    description: String,
    sculpt: f64,
    vibe: f64,
    operator: String,
    min_delta: Option<f64>,
}

#[derive(Debug, serde::Deserialize)]
struct DataHeavyGateSpec {
    name: String,
    source_metrics: String,
    #[serde(default)]
    thresholds: DataHeavyGateThresholds,
    #[serde(default)]
    observed: Option<DataHeavyGateObserved>,
    #[serde(default)]
    criteria: Option<DataHeavyGateObserved>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DataHeavyGateObserved {
    acceptance_rate: f64,
    repro_pass: usize,
    repro_unique_hashes: usize,
    reproducible: bool,
    #[serde(default)]
    infra_blocked: bool,
    #[serde(default)]
    infra_failures: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DataHeavyGateThresholds {
    min_acceptance_rate: f64,
    min_repro_pass: usize,
    max_repro_unique_hashes: usize,
    require_reproducible: bool,
}

impl Default for DataHeavyGateThresholds {
    fn default() -> Self {
        Self {
            min_acceptance_rate: 1.0,
            min_repro_pass: 5,
            max_repro_unique_hashes: 1,
            require_reproducible: true,
        }
    }
}

fn write_examples() -> Result<()> {
    let examples_dir = Path::new("examples");
    fs::create_dir_all(examples_dir)?;

    let files: &[(&str, &str)] = &[
        (
            "getting-started/hello_world.sculpt",
            r#"# Hello World (tradition kept)
# Minimal deterministic example with no ND.

@meta target=cli

module(HelloWorld):

  flow(App):
    start > Show

    state(Show):
      render text("Hallo", color: "yellow")
      render text("Welt", color: "blue")
      on key(Esc) > Exit
    end

    state(Exit):
      terminate
    end
  end

end
"#,
        ),
        (
            "getting-started/native_window.sculpt",
            r#"# Native Window Demo (macOS GUI)
# Goal: show a real window with text + button.

@meta target=gui

module(NativeWindow):

  flow(App):
    start > Main

    state(Main):
      render text("SCULPT Native Demo", color: "yellow")
      render text("Click the button to open an OK modal", color: "blue")
      render button("Open OK", action: "modal.ok")
      on key(Esc) > Exit
    end

    state(Exit):
      terminate
    end
  end

end
"#,
        ),
        (
            "games/snake_high_nd.sculpt",
            r#"# Snake (High ND)
# Goal: minimal code, large solution space.
# Most of the game design is delegated to the LLM.

@meta target=cli

module(SnakeHighND):

  # Main flow
  flow(Game):
    start > Title

    state(Title):
      render text("SNAKE", color: "yellow")
      render text("Press Enter", color: "blue")
      on key(Enter) > Play
      on key(Esc)   > Exit
    end

    state(Play):
      on tick > Play
      on done > Title
      on key(Esc) > Exit
    end

    state(Exit):
      terminate
    end
  end

  # Minimal state (intentionally sparse)
  state():
    speedMs = 160
    score = 0
  end

  rule(tick):
    on tick:
      score += 1
    end
  end

  rule(finish):
    when score >= 10:
      emit done
    end
  end

  # High-ND block: most of the game definition is delegated to the LLM
  nd(designSnake):
    propose game("snake", size: 10)
    satisfy(
      playable(),
      funToLearn(),
      usesKeys("WASD"),
      loopedGameplay()
    )
  end
end
"#,
        ),
        (
            "games/snake_low_nd.sculpt",
            r#"# Snake (Low ND)
# Goal: highly specified rules so the solution space is narrow.
# ND is reduced to a tiny UI-theme choice.

@meta target=cli

module(SnakeLowND):

  # Main flow
  flow(Game):
    start > Title

    state(Title):
      render text("SNAKE", color: "yellow")
      render text("Enter = Start, Esc = Quit", color: "blue")
      on key(Enter) > Play
      on key(Esc)   > Exit
    end

    state(Play):
      on tick > Play
      on done > Title
      on key(Esc) > Exit
    end

    state(Exit):
      terminate
    end
  end

  # Deterministic configuration and initial game state
  state():
    width = 16
    height = 12
    speedMs = 120
    score = 0
    direction = "right"
    pendingDirection = "right"
    snake = "[(8,6),(7,6),(6,6)]"   # head first
    food = "(12,6)"
    foodSequence = "[(12,6),(12,7),(11,7),(10,7),(10,6)]"
    stepIndex = 0
    wallHit = 0
    selfHit = 0
  end

  # Deterministic tick progression
  rule(tick):
    on tick:
      score += 1
      stepIndex += 1
    end
  end

  # Deterministic input handling
  rule(inputUp):
    on key(W):
      pendingDirection = "up"
    end
  end

  rule(inputDown):
    on key(S):
      pendingDirection = "down"
    end
  end

  rule(inputLeft):
    on key(A):
      pendingDirection = "left"
    end
  end

  rule(inputRight):
    on key(D):
      pendingDirection = "right"
    end
  end

  # Deterministic movement and food handling
  rule(applyDirection):
    on tick:
      direction = pendingDirection
      snake = moveSnake(snake, direction)
      food = nextFood(foodSequence, stepIndex)
    end
  end

  rule(checkCollision):
    on tick:
      wallHit = hitWall(snake, width, height)
      selfHit = hitSelf(snake)
    end
  end

  rule(collisionWall):
    when wallHit >= 1:
      emit done
    end
  end

  rule(collisionSelf):
    when selfHit >= 1:
      emit done
    end
  end

  # Deterministic finish condition
  rule(finish):
    when score >= 25:
      emit done
    end
  end

  # Tiny ND for cosmetics only
  nd(theme):
    propose theme("classic")
    satisfy(
      exact("classic"),
      highContrast()
    )
  end
end
"#,
        ),
        (
            "games/breakout_low_nd.sculpt",
            r#"# Breakout (CLI)
# Demonstrates a playable arcade loop with clear game-state rules and constrained ND for level layout.

@meta target=cli
@meta nd_budget=24
@meta confidence=0.9

module(BreakoutLowND):

  flow(Game):
    start > Title

    state(Title):
      render text("BREAKOUT", color: "yellow")
      render text("A/D move paddle, Space launch, Esc quit", color: "blue")
      on key(Enter) > Play
      on key(Esc) > Exit
    end

    state(Play):
      on tick > Play
      on done > GameOver
      on key(Esc) > Exit

      on key(Space):: launched = 1
      on key(A):: paddleX = movePaddle(paddleX, "left", width, paddleWidth)
      on key(D):: paddleX = movePaddle(paddleX, "right", width, paddleWidth)

      rule(runtimeTick):
        on tick:
          score += 1
          hitLeft = detectHitLeft(ball)
          hitRight = detectHitRight(ball, width)
          hitTop = detectHitTop(ball)
          hitPaddle = detectHitPaddle(ball, paddleX, paddleWidth)
          hitBottom = detectHitBottom(ball, height)
        end
      end

      rule(runtimeFinish):
        when hitBottom >= 1 and lives < 1:
          emit done
        end
      end
    end

    state(GameOver):
      render text("Game Over", color: "red")
      render text("Enter restart, Esc quit", color: "blue")
      on key(Enter) > Title
      on key(Esc) > Exit
    end

    state(Exit):
      terminate
    end
  end

  state():
    width = 40
    height = 22
    paddleX = 18
    paddleWidth = 6
    ball = "(20,16)"
    velocity = "up_right"
    launched = 0
    score = 0
    lives = 3
    bricks = "rows:5 cols:10 pattern:solid"
    speedMs = 65
    hitLeft = 0
    hitRight = 0
    hitTop = 0
    hitPaddle = 0
    hitBottom = 0
  end

  rule(tick):
    on tick:
      ball = stepBallIfLaunched(ball, velocity, launched)
      score += 1
      hitLeft = detectHitLeft(ball)
      hitRight = detectHitRight(ball, width)
      hitTop = detectHitTop(ball)
      hitPaddle = detectHitPaddle(ball, paddleX, paddleWidth)
      hitBottom = detectHitBottom(ball, height)
    end
  end

  rule(wallBounceLeft):
    when hitLeft >= 1:
      velocity = bounceX(velocity)
    end
  end

  rule(wallBounceRight):
    when hitRight >= 1:
      velocity = bounceX(velocity)
    end
  end

  rule(wallBounceTop):
    when hitTop >= 1:
      velocity = bounceY(velocity)
    end
  end

  rule(paddleBounce):
    when hitPaddle >= 1:
      velocity = bounceY(velocity)
    end
  end

  rule(bottomOut):
    when hitBottom >= 1:
      lives = dec(lives)
      launched = 0
      ball = resetBallNearPaddle(paddleX)
      velocity = "up_right"
    end
  end

  rule(finish):
    when lives < 1:
      emit done
    end
  end

  nd(levelLayout):
    propose brickLayout(rows: 5, cols: 10, style: "classic")
    satisfy(
      fullyInsideBounds(width: 40, height: 12),
      symmetricStart(),
      reachableByBallPhysics(),
      progressiveDifficulty()
    )
  end

end
"#,
        ),
        (
            "business/invoice_review.sculpt",
            r#"# Business/Web Example: Invoice Review
# Goal: clear business UI with minimal ND.

@meta target=cli

module(InvoiceReview):

  flow(App):
    start > List

    state(List):
      render text("Invoices", color: "yellow")
      render text("Enter = Open First, Esc = Quit", color: "blue")
      on key(Enter) > Detail
      on key(Esc)   > Exit
    end

    state(Detail):
      render text("Invoice #2024-001", color: "yellow")
      render text("Amount: 1,250 EUR", color: "blue")
      render text("Status: Pending", color: "blue")
      on key(A) > Approve
      on key(R) > Reject
      on key(Esc) > List
    end

    state(Approve):
      render text("Approved", color: "green")
      on key(Enter) > List
    end

    state(Reject):
      render text("Rejected", color: "red")
      on key(Enter) > List
    end

    state(Exit):
      terminate
    end
  end

  state():
    speedMs = 400
    selectedInvoice = "2024-001"
    totalInvoices = 24
  end

  # Tiny ND: layout theme only, business logic is explicit
  nd(theme):
    propose dashboardTheme("clean")
    satisfy(
      highContrast(),
      professionalTone()
    )
  end
end
"#,
        ),
        (
            "business/incident_triage_assistant.sculpt",
            r#"# Incident Triage Assistant (PoC)
# Real-world task: guide on-call engineers to a first-response action plan.

@meta target=cli
@meta nd_budget=30
@meta confidence=0.85

module(Ops.Incident.Triage):

  flow(Main):
    start > Intro

    state(Intro):
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

    state(ServiceDown):
      render text("SERVICE DOWN", color: "red")
      render text("Action plan:", color: "yellow")
      render text("- Declare SEV-1", color: "white")
      render text("- Start status page incident", color: "white")
      render text("- Assign commander + comms owner", color: "white")
      render text("- Roll back latest deploy if recent", color: "white")
      render text("Enter = Back", color: "blue")
      on key(enter) > Intro
    end

    state(ErrorSpike):
      render text("ERROR SPIKE", color: "magenta")
      render text("Action plan:", color: "yellow")
      render text("- Check top failing endpoint", color: "white")
      render text("- Compare release/version deltas", color: "white")
      render text("- Enable degraded mode if available", color: "white")
      render text("- Page owning team if >5 min sustained", color: "white")
      render text("Enter = Back", color: "blue")
      on key(enter) > Intro
    end

    state(Latency):
      render text("LATENCY INCREASE", color: "cyan")
      render text("Action plan:", color: "yellow")
      render text("- Check DB and cache saturation", color: "white")
      render text("- Inspect queue backlog", color: "white")
      render text("- Apply temporary rate-limit if needed", color: "white")
      render text("- Capture flamegraph before restart", color: "white")
      render text("Enter = Back", color: "blue")
      on key(enter) > Intro
    end

    state(Exit):
      terminate
    end
  end

  # Constrain wording and structure for lower ND.
  nd(incidentPlaybookShape):
    propose responseGuide(format: "step-list", audience: "on-call")
    satisfy(
      hasClearTitle(),
      hasActionableSteps(min: 4),
      usesOperationalLanguage(),
      supportsQuickKeyNavigation()
    )
  end

end
"#,
        ),
        (
            "business/expense_approval_workflow.sculpt",
            r#"# Expense Approval Workflow (Business)
# Demonstrates a real approval flow where logic is explicit and ND is limited to message tone.

@meta target=cli
@meta nd_budget=8

module(Business.Finance.ExpenseApproval):

  flow(App):
    start > Inbox

    state(Inbox):
      render text("Expense Queue", color: "yellow")
      render text("1 = Open request, Esc = Exit", color: "blue")
      on key(1) > Review
      on key(Esc) > Exit
    end

    state(Review):
      render text("Request #EX-2048", color: "yellow")
      render text("Amount: 890 EUR", color: "white")
      render text("Category: Travel", color: "white")
      render text("A = Approve, R = Reject, N = Need Info", color: "blue")
      on key(A):: approvedCount += 1
      on key(R):: rejectedCount += 1
      on key(N):: infoCount += 1
      on key(A) > Approved
      on key(R) > Rejected
      on key(N) > NeedInfo
      on key(Esc) > Inbox

      rule(autoEscalate):
        when riskScore >= 70 and queueSize > 10 or amount == 5000:
          emit escalated
        end
      end

      on escalated > NeedInfo
    end

    state(Approved):
      render text("Approved and forwarded to payout batch.", color: "green")
      on key(Enter) > Inbox
    end

    state(Rejected):
      render text("Rejected with reason code.", color: "red")
      on key(Enter) > Inbox
    end

    state(NeedInfo):
      render text("Sent request for additional receipt details.", color: "magenta")
      on key(Enter) > Inbox
    end

    state(Exit):
      terminate
    end
  end

  state():
    amount = 890
    approvalLimit = 1000
    queueSize = 12
    selectedRequest = "EX-2048"
    riskScore = 18
    approvedCount = 0
    rejectedCount = 0
    infoCount = 0
  end

  rule(policyCheck):
    when riskScore >= 80:
      emit done
    end
  end

  nd(copyStyle):
    propose reviewCopyTone(audience: "finance-team")
    satisfy(
      conciseLanguage(),
      noLegalRiskTerms(),
      professionalTone()
    )
  end

end
"#,
        ),
        (
            "web/incident_status_dashboard.sculpt",
            r#"# Incident Status Dashboard (Web)
# Demonstrates a web target with strongly defined flow and constrained ND for layout only.

@meta target=web
@meta nd_budget=12

module(Ops.Web.IncidentDashboard):

  flow(App):
    start > Overview

    state(Overview):
      render text("Incident Status Dashboard", color: "yellow")
      render text("1 = Active incidents, 2 = Timeline, Esc = Exit", color: "blue")
      on key(1) > ActiveIncidents
      on key(2) > Timeline
      on key(Esc) > Exit
    end

    state(ActiveIncidents):
      render text("SEV-1 Checkout API", color: "red")
      render text("SEV-2 Search Latency", color: "magenta")
      render text("Enter = Back", color: "blue")
      on key(Enter) > Overview
    end

    state(Timeline):
      render text("14:02 deploy started", color: "white")
      render text("14:07 error rate crossed SLO", color: "white")
      render text("14:10 rollback started", color: "white")
      render text("Enter = Back", color: "blue")
      on key(Enter) > Overview
    end

    state(Exit):
      terminate
    end
  end

  state():
    refreshSec = 15
    showResolved = 0
    selectedIncident = "checkout-api"
  end

  rule(autoRefresh):
    on tick:
      refreshSec += 0
    end
  end

  nd(layout):
    propose dashboardLayout(kind: "ops", density: "medium")
    satisfy(
      noOverlap(),
      clearSeverityHierarchy(),
      keyboardNavigable(),
      mobileFallbackExists()
    )
  end

end
"#,
        ),
        (
            "web/support_ticket_board.sculpt",
            r#"# Support Ticket Board (Web)
# Demonstrates a small but practical web workflow with multiple screens and keyboard navigation.

@meta target=web
@meta nd_budget=10

module(ServiceDesk.Web.SupportBoard):

  flow(App):
    start > Board

    state(Board):
      render text("Support Ticket Board", color: "yellow")
      render text("1 = Open ticket #4821", color: "white")
      render text("2 = Open ticket #4822", color: "white")
      render text("S = SLA overview", color: "blue")
      render text("Esc = Exit", color: "blue")
      on key(1) > Ticket4821
      on key(2) > Ticket4822
      on key(S) > SLA
      on key(Esc) > Exit
    end

    state(Ticket4821):
      render text("Ticket #4821", color: "yellow")
      render text("Customer: ACME Retail", color: "white")
      render text("Issue: Checkout timeout", color: "white")
      render text("Priority: High", color: "red")
      render text("A = Mark in progress, R = Return", color: "blue")
      on key(A) > InProgress
      on key(R) > Board
      on key(Esc) > Exit
    end

    state(Ticket4822):
      render text("Ticket #4822", color: "yellow")
      render text("Customer: Nova Health", color: "white")
      render text("Issue: CSV export mismatch", color: "white")
      render text("Priority: Medium", color: "magenta")
      render text("A = Mark in progress, R = Return", color: "blue")
      on key(A) > InProgress
      on key(R) > Board
      on key(Esc) > Exit
    end

    state(InProgress):
      render text("Ticket moved to In Progress.", color: "green")
      render text("Enter = Back to board", color: "blue")
      on key(Enter) > Board
      on key(Esc) > Exit
    end

    state(SLA):
      render text("SLA Overview", color: "yellow")
      render text("High: 1 overdue", color: "red")
      render text("Medium: 2 due in < 4h", color: "magenta")
      render text("Low: 6 on track", color: "green")
      render text("Enter = Back to board", color: "blue")
      on key(Enter) > Board
      on key(Esc) > Exit
    end

    state(Exit):
      terminate
    end
  end

  nd(layout):
    propose boardLayout(kind: "service-desk")
    satisfy(
      clearPriorityContrast(),
      keyboardFirstNavigation(),
      readableOnLaptopScreens()
    )
  end

end
"#,
        ),
    ];

    for (relative, content) in files {
        let path = examples_dir.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        println!("Wrote {}", path.display());
    }
    Ok(())
}

fn project_create(name: &str, path: Option<&Path>, files: Option<&[String]>) -> Result<()> {
    let base_dir = path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| env::current_dir().expect("current dir"));
    if !base_dir.exists() {
        bail!("Path does not exist: {}", base_dir.display());
    }
    if !base_dir.is_dir() {
        bail!("Path is not a directory: {}", base_dir.display());
    }

    let patterns: Vec<String> = files
        .map(|v| v.to_vec())
        .unwrap_or_else(|| vec!["*.sculpt".to_string()]);

    let module_files = resolve_module_files(&base_dir, &patterns)?;
    if module_files.is_empty() {
        bail!(
            "No .sculpt files found for patterns: {}",
            patterns.join(", ")
        );
    }

    let mut entry_module = None;
    let mut rel_modules = Vec::new();
    for file in &module_files {
        let source = fs::read_to_string(file)
            .with_context(|| format!("Failed to read {}", file.display()))?;
        let module =
            parse_source(&source).with_context(|| format!("Failed to parse {}", file.display()))?;
        if entry_module.is_none() {
            entry_module = Some(module.name.clone());
        }
        let rel = file
            .strip_prefix(&base_dir)
            .unwrap_or(file.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        rel_modules.push(rel);
    }

    let entry = entry_module.ok_or_else(|| anyhow::anyhow!("No module entry found"))?;
    let project_name = normalize_project_name(name);
    let project_filename = if name.ends_with(".sculpt.json") {
        name.to_string()
    } else {
        format!("{name}.sculpt.json")
    };
    let project_file = base_dir.join(project_filename);

    let json = serde_json::json!({
        "name": project_name,
        "entry": entry,
        "modules": rel_modules,
    });
    let text = serde_json::to_string_pretty(&json)?;
    fs::write(&project_file, format!("{text}\n"))
        .with_context(|| format!("Failed to write {}", project_file.display()))?;

    println!("Project file created: {}", project_file.display());
    Ok(())
}

fn resolve_module_files(base_dir: &Path, patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for pattern in patterns {
        let has_glob = pattern.contains('*')
            || pattern.contains('?')
            || pattern.contains('[')
            || pattern.contains(']');
        if has_glob {
            let pat_path = if Path::new(pattern).is_absolute() {
                PathBuf::from(pattern)
            } else {
                base_dir.join(pattern)
            };
            let pat = pat_path.to_string_lossy().to_string();
            for entry in glob(&pat).with_context(|| format!("Invalid glob pattern: {pattern}"))? {
                let file =
                    entry.with_context(|| format!("Bad glob match for pattern: {pattern}"))?;
                if file.is_file()
                    && file.extension().and_then(|s| s.to_str()) == Some("sculpt")
                    && seen.insert(file.clone())
                {
                    out.push(file);
                }
            }
        } else {
            let file = if Path::new(pattern).is_absolute() {
                PathBuf::from(pattern)
            } else {
                base_dir.join(pattern)
            };
            if !file.exists() {
                bail!("File not found: {}", file.display());
            }
            if file.extension().and_then(|s| s.to_str()) != Some("sculpt") {
                bail!("Not a .sculpt file: {}", file.display());
            }
            if seen.insert(file.clone()) {
                out.push(file);
            }
        }
    }

    out.sort();
    Ok(out)
}

fn normalize_project_name(raw: &str) -> String {
    raw.trim_end_matches(".sculpt.json")
        .trim_end_matches(".json")
        .to_string()
}

#[derive(Clone, Copy)]
enum DebugLevel {
    Compact,
    Raw,
    All,
    Json,
}

fn parse_debug(level: Option<String>) -> Option<DebugLevel> {
    let Some(value) = level else {
        return None;
    };
    match value.as_str() {
        "compact" => Some(DebugLevel::Compact),
        "raw" => Some(DebugLevel::Raw),
        "all" => Some(DebugLevel::All),
        "json" => Some(DebugLevel::Json),
        _ => None,
    }
}

fn build(
    input: &Path,
    target: Option<&str>,
    nd_policy_override: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    strict: bool,
    debug: Option<String>,
) -> Result<()> {
    let started = Instant::now();
    let ir = load_ir(input, nd_policy_override.as_deref())?;
    let controls = ConvergenceControls::from_meta(&ir.meta);
    let target = resolve_target_from_meta(target, &ir)?;
    let layout_required = enforce_meta(&ir, &target)?;
    let ir_json = to_pretty_json(&ir)?;
    let nondet = generate_report(&ir);

    let dist_dir = dist_dir(input);
    fs::create_dir_all(&dist_dir)?;
    fs::write(dist_dir.join("ir.json"), ir_json)?;
    fs::write(dist_dir.join("nondet.report"), &nondet)?;

    let target_descriptor = describe_target(&target)?;
    let contract = parse_target_contract(&target_descriptor)?;
    validate_module_against_contract(&ir, &target, &contract)?;
    let spec = build_target_spec_from_value(&target_descriptor)?;
    let debug_level = parse_debug(debug);
    let (ai_provider, provider_info) = select_ai_provider(provider.clone(), model.clone(), strict)?;
    print_unified_header("Build", &target, input, Some(&provider_info));
    print_step("1", "Parse & Validate", "ok");
    let sculpt_ir_value = serde_json::to_value(&ir)?;
    let previous_target_ir = read_previous_target_ir(input);

    let spinner = start_spinner("2", "LLM Compile");
    let target_ir_result = generate_with_convergence(
        ai_provider,
        provider.clone(),
        model.clone(),
        strict,
        &sculpt_ir_value,
        &spec,
        &nondet,
        previous_target_ir.as_ref(),
        layout_required,
        &controls,
    );
    stop_spinner(spinner);
    if target_ir_result.is_ok() {
        finish_step("2", "LLM Compile", "ok");
    } else {
        finish_step("2", "LLM Compile", "failed");
    }
    let (target_ir_value, debug_capture) = target_ir_result?;
    let target_ir = match from_json_value(target_ir_value.clone()) {
        Ok(ir) => ir,
        Err(e) => {
            if let Some(level) = debug_level {
                eprintln!("Debug (parse failure):");
                eprintln!("  target={} input={}", target, input.display());
                eprintln!("  standard_ir={}", spec.standard_ir);
                if matches!(level, DebugLevel::Raw | DebugLevel::All) {
                    if let Some(c) = debug_capture.as_ref() {
                        eprintln!("--- raw output ---");
                        eprintln!("{}", c.raw_output);
                    }
                }
                if matches!(level, DebugLevel::All | DebugLevel::Json) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&target_ir_value) {
                        eprintln!("--- normalized target ir ---");
                        eprintln!("{}", pretty);
                    }
                }
            }
            return Err(anyhow::anyhow!("Target IR parse error: {}", e));
        }
    };
    if target_ir.ir_type != spec.standard_ir {
        bail!(
            "Target IR type mismatch: expected {}, got {}",
            spec.standard_ir,
            target_ir.ir_type
        );
    }
    if layout_required && target_ir.layout.is_none() {
        bail!("layout=explicit requires layout data in target IR");
    }
    validate_required_output_contract(&ir, &target_ir, &target)?;

    fs::write(
        dist_dir.join("target.ir.json"),
        serde_json::to_string_pretty(&target_ir_value)?,
    )?;
    let spinner = start_spinner("3", "Build Target");
    let build_started = Instant::now();
    let build_result = deterministic_build(&target, &target_ir, &target_ir_value, input, &dist_dir);
    let build_ms = build_started.elapsed().as_millis();
    stop_spinner(spinner);
    if let Err(e) = build_result {
        finish_step("3", "Build Target", "failed");
        return Err(e);
    }
    finish_step("3", "Build Target", "ok");
    verify_build_artifacts(&target, &dist_dir)?;

    if let Some(level) = debug_level {
        emit_debug(
            level,
            &target,
            input,
            &provider_info,
            &spec,
            &target_ir,
            debug_capture.as_ref(),
            build_ms,
        );
    }

    let total_ms = started.elapsed().as_millis();
    let llm_ms = debug_capture.as_ref().map(|c| c.llm_ms);
    let token_usage = debug_capture.as_ref().and_then(|c| c.token_usage.clone());
    write_build_meta(
        &dist_dir,
        &BuildMeta {
            version: 1,
            script: input.display().to_string(),
            action: "build".to_string(),
            target: target.clone(),
            provider: Some(provider_info.name.clone()),
            model: Some(provider_info.model.clone()),
            llm_ms,
            build_ms: Some(build_ms),
            run_ms: None,
            total_ms,
            timestamp_unix_ms: now_unix_ms(),
            status: "ok".to_string(),
            token_usage: token_usage.clone(),
        },
    )?;
    maybe_auto_clean_dist(&dist_dir);

    print_unified_footer(
        &[
            &format!("{}/target.ir.json", dist_dir.display()),
            &format!("{}/ir.json", dist_dir.display()),
            &format!("{}/nondet.report", dist_dir.display()),
        ],
        token_usage.as_ref(),
    );
    Ok(())
}

fn freeze(
    input: &Path,
    nd_policy_override: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    strict: bool,
    target: Option<&str>,
    debug: Option<String>,
) -> Result<()> {
    let started = Instant::now();
    let ir = load_ir(input, nd_policy_override.as_deref())?;
    let controls = ConvergenceControls::from_meta(&ir.meta);
    let target = resolve_target_from_meta(target, &ir)?;
    let layout_required = enforce_meta(&ir, &target)?;
    let nondet = generate_report(&ir);

    let target_descriptor = describe_target(&target)?;
    let contract = parse_target_contract(&target_descriptor)?;
    validate_module_against_contract(&ir, &target, &contract)?;
    let spec = build_target_spec_from_value(&target_descriptor)?;
    let debug_level = parse_debug(debug);
    let (ai_provider, provider_info) = select_ai_provider(provider.clone(), model.clone(), strict)?;
    print_unified_header("Freeze", &target, input, Some(&provider_info));
    print_step("1", "Parse & Validate", "ok");
    let sculpt_ir_value = serde_json::to_value(&ir)?;
    let previous_target_ir = read_previous_target_ir(input);

    let spinner = start_spinner("2", "LLM Compile");
    let target_ir_result = generate_with_convergence(
        ai_provider,
        provider.clone(),
        model.clone(),
        strict,
        &sculpt_ir_value,
        &spec,
        &nondet,
        previous_target_ir.as_ref(),
        layout_required,
        &controls,
    );
    stop_spinner(spinner);
    if target_ir_result.is_ok() {
        finish_step("2", "LLM Compile", "ok");
    } else {
        finish_step("2", "LLM Compile", "failed");
    }
    let (target_ir_value, debug_capture) = target_ir_result?;
    let target_ir = match from_json_value(target_ir_value.clone()) {
        Ok(ir) => ir,
        Err(e) => {
            if let Some(level) = debug_level {
                eprintln!("Debug (parse failure):");
                eprintln!("  target={} input={}", target, input.display());
                eprintln!("  standard_ir={}", spec.standard_ir);
                if matches!(level, DebugLevel::Raw | DebugLevel::All) {
                    if let Some(c) = debug_capture.as_ref() {
                        eprintln!("--- raw output ---");
                        eprintln!("{}", c.raw_output);
                    }
                }
                if matches!(level, DebugLevel::All | DebugLevel::Json) {
                    if let Ok(pretty) = serde_json::to_string_pretty(&target_ir_value) {
                        eprintln!("--- normalized target ir ---");
                        eprintln!("{}", pretty);
                    }
                }
            }
            return Err(anyhow::anyhow!("Target IR parse error: {}", e));
        }
    };
    if target_ir.ir_type != spec.standard_ir {
        bail!(
            "Target IR type mismatch: expected {}, got {}",
            spec.standard_ir,
            target_ir.ir_type
        );
    }
    if layout_required && target_ir.layout.is_none() {
        bail!("layout=explicit requires layout data in target IR");
    }
    validate_required_output_contract(&ir, &target_ir, &target)?;

    let lock = create_lock(
        &ir,
        &provider_info.name,
        &target,
        &target_ir_value,
        &provider_info.model,
    )?;
    write_lock(Path::new("sculpt.lock"), &lock)?;

    let dist_dir = dist_dir(input);
    fs::create_dir_all(&dist_dir)?;
    fs::write(
        dist_dir.join("target.ir.json"),
        serde_json::to_string_pretty(&target_ir_value)?,
    )?;
    fs::write(dist_dir.join("ir.json"), to_pretty_json(&ir)?)?;
    fs::write(dist_dir.join("nondet.report"), &nondet)?;

    let spinner = start_spinner("3", "Build Target");
    let build_started = Instant::now();
    let build_result = deterministic_build(&target, &target_ir, &target_ir_value, input, &dist_dir);
    let build_ms = build_started.elapsed().as_millis();
    stop_spinner(spinner);
    if let Err(e) = build_result {
        finish_step("3", "Build Target", "failed");
        return Err(e);
    }
    finish_step("3", "Build Target", "ok");
    verify_build_artifacts(&target, &dist_dir)?;

    if let Some(level) = debug_level {
        emit_debug(
            level,
            &target,
            input,
            &provider_info,
            &spec,
            &target_ir,
            debug_capture.as_ref(),
            build_ms,
        );
    }

    let total_ms = started.elapsed().as_millis();
    let llm_ms = debug_capture.as_ref().map(|c| c.llm_ms);
    let token_usage = debug_capture.as_ref().and_then(|c| c.token_usage.clone());
    write_build_meta(
        &dist_dir,
        &BuildMeta {
            version: 1,
            script: input.display().to_string(),
            action: "freeze".to_string(),
            target: target.clone(),
            provider: Some(provider_info.name.clone()),
            model: Some(provider_info.model.clone()),
            llm_ms,
            build_ms: Some(build_ms),
            run_ms: None,
            total_ms,
            timestamp_unix_ms: now_unix_ms(),
            status: "ok".to_string(),
            token_usage: token_usage.clone(),
        },
    )?;
    maybe_auto_clean_dist(&dist_dir);

    print_unified_footer(
        &[
            "sculpt.lock",
            &format!("{}/target.ir.json", dist_dir.display()),
            &format!("{}/ir.json", dist_dir.display()),
            &format!("{}/nondet.report", dist_dir.display()),
        ],
        token_usage.as_ref(),
    );
    Ok(())
}

fn replay(input: &Path, target: Option<&str>) -> Result<()> {
    let started = Instant::now();
    let ir = load_ir(input, None)?;
    let target = resolve_target_from_meta(target, &ir)?;
    let layout_required = enforce_meta(&ir, &target)?;
    print_unified_header("Replay", &target, input, None);
    print_step("1", "Parse & Validate", "ok");
    let lock = read_lock(Path::new("sculpt.lock"))?;
    verify_lock(&ir, &lock)?;

    print_step("2", "Load Lock", "ok");
    let target_ir_value = lock.target_ir.clone();
    let target_ir = from_json_value(target_ir_value.clone())
        .map_err(|e| anyhow::anyhow!("Target IR parse error: {}", e))?;
    if layout_required && target_ir.layout.is_none() {
        bail!("layout=explicit requires layout data in target IR");
    }
    validate_required_output_contract(&ir, &target_ir, &target)?;

    let dist_dir = dist_dir(input);
    fs::create_dir_all(&dist_dir)?;
    fs::write(
        dist_dir.join("target.ir.json"),
        serde_json::to_string_pretty(&target_ir_value)?,
    )?;
    let spinner = start_spinner("3", "Build Target");
    let build_result = deterministic_build(&target, &target_ir, &target_ir_value, input, &dist_dir);
    stop_spinner(spinner);
    if let Err(e) = build_result {
        finish_step("3", "Build Target", "failed");
        return Err(e);
    }
    finish_step("3", "Build Target", "ok");
    verify_build_artifacts(&target, &dist_dir)?;
    let total_ms = started.elapsed().as_millis();
    write_build_meta(
        &dist_dir,
        &BuildMeta {
            version: 1,
            script: input.display().to_string(),
            action: "replay".to_string(),
            target: target.clone(),
            provider: Some("replay".to_string()),
            model: Some("locked".to_string()),
            llm_ms: None,
            build_ms: None,
            run_ms: None,
            total_ms,
            timestamp_unix_ms: now_unix_ms(),
            status: "ok".to_string(),
            token_usage: None,
        },
    )?;
    maybe_auto_clean_dist(&dist_dir);
    fs::write(dist_dir.join("ir.json"), to_pretty_json(&ir)?)?;
    fs::write(dist_dir.join("nondet.report"), generate_report(&ir))?;

    print_unified_footer(
        &[
            &format!("{}/target.ir.json", dist_dir.display()),
            &format!("{}/ir.json", dist_dir.display()),
            &format!("{}/nondet.report", dist_dir.display()),
        ],
        None,
    );
    Ok(())
}

fn emit_debug(
    level: DebugLevel,
    target: &str,
    input: &Path,
    provider: &ProviderInfo,
    spec: &TargetSpec,
    target_ir: &TargetIr,
    capture: Option<&DebugCapture>,
    build_ms: u128,
) {
    let llm_ms = capture.map(|c| c.llm_ms).unwrap_or(0);
    let view_count = target_ir.views.len();
    let transition_count: usize = target_ir.flow.transitions.values().map(|m| m.len()).sum();

    let mut out = serde_json::json!({
      "provider": provider.name,
      "model": provider.model,
      "target": target,
      "input": input,
      "standard_ir": spec.standard_ir,
      "summary": {
        "flow_start": target_ir.flow.start,
        "views": view_count,
        "transitions": transition_count
      },
      "timing_ms": {
        "llm": llm_ms,
        "build": build_ms
      },
      "outputs": [
        &format!("{}/target.ir.json", dist_dir(input).display()),
        &format!("{}/ir.json", dist_dir(input).display()),
        &format!("{}/nondet.report", dist_dir(input).display())
      ]
    });

    if matches!(level, DebugLevel::Raw | DebugLevel::All | DebugLevel::Json) {
        if let Some(c) = capture {
            out["raw_output"] = serde_json::Value::String(c.raw_output.clone());
        }
    }

    if matches!(level, DebugLevel::All | DebugLevel::Json) {
        if let Some(c) = capture {
            out["prompt"] = serde_json::Value::String(c.prompt.clone());
        }
    }

    if matches!(level, DebugLevel::Json) {
        eprintln!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        return;
    }

    eprintln!("Debug:");
    eprintln!("  provider={} model={}", provider.name, provider.model);
    eprintln!("  target={} input={}", target, input.display());
    eprintln!("  standard_ir={}", spec.standard_ir);
    eprintln!(
        "  summary: start={} views={} transitions={}",
        target_ir.flow.start, view_count, transition_count
    );
    eprintln!("  timing_ms: llm={} build={}", llm_ms, build_ms);
    eprintln!(
        "  outputs: {}/target.ir.json {}/ir.json {}/nondet.report",
        dist_dir(input).display(),
        dist_dir(input).display(),
        dist_dir(input).display()
    );

    if matches!(level, DebugLevel::Raw | DebugLevel::All) {
        if let Some(c) = capture {
            eprintln!("--- raw output ---");
            eprintln!("{}", c.raw_output);
        }
    }
    if matches!(level, DebugLevel::All) {
        if let Some(c) = capture {
            eprintln!("--- prompt ---");
            eprintln!("{}", c.prompt);
        }
    }
}

fn run_cmd(input: &Path, target: Option<&str>) -> Result<()> {
    let started = Instant::now();
    let ir = load_ir(input, None)?;
    let target = resolve_target_from_meta(target, &ir)?;
    print_unified_header("Run", &target, input, None);
    let dist_dir = dist_dir(input);
    let result = match resolve_target(&target) {
        TargetKind::Cli => run_cli(&dist_dir),
        TargetKind::Web => run_web(&dist_dir),
        TargetKind::Gui => run_gui(&dist_dir),
        TargetKind::External(name) => {
            run_external_target(&name, &ir, None, None, &dist_dir, input, None, "run")?;
            Ok(())
        }
    };
    result?;
    verify_required_outputs(&ir)?;
    let run_ms = started.elapsed().as_millis();
    let meta = BuildMeta {
        version: 1,
        script: input.display().to_string(),
        action: "run".to_string(),
        target: target.clone(),
        provider: None,
        model: None,
        llm_ms: None,
        build_ms: None,
        run_ms: Some(run_ms),
        total_ms: run_ms,
        timestamp_unix_ms: now_unix_ms(),
        status: "ok".to_string(),
        token_usage: None,
    };
    write_build_meta(&dist_dir, &meta)?;
    maybe_auto_clean_dist(&dist_dir);
    Ok(())
}

fn verify_build_artifacts(target: &str, dist_dir: &Path) -> Result<()> {
    let core_files = [
        dist_dir.join("target.ir.json"),
        dist_dir.join("ir.json"),
        dist_dir.join("nondet.report"),
    ];
    for f in core_files {
        if !f.exists() {
            bail!("Build artifact missing: {}", f.display());
        }
    }

    match resolve_target(target) {
        TargetKind::Cli => {
            let entry = dist_dir.join("main.js");
            if !entry.exists() {
                bail!("Build artifact missing: {}", entry.display());
            }
        }
        TargetKind::Web => {
            let entry_html = dist_dir.join("index.html");
            let entry_js = dist_dir.join("main.js");
            if !entry_html.exists() {
                bail!("Build artifact missing: {}", entry_html.display());
            }
            if !entry_js.exists() {
                bail!("Build artifact missing: {}", entry_js.display());
            }
        }
        TargetKind::Gui => {
            let native_macos = dist_dir.join("gui").join(".build").join("release").join("SculptGui");
            let python_entry = dist_dir.join("gui").join("main.py");
            if !native_macos.exists() && !python_entry.exists() {
                bail!(
                    "Build artifact missing: expected {} or {}",
                    native_macos.display(),
                    python_entry.display()
                );
            }
        }
        TargetKind::External(_) => {}
    }
    Ok(())
}

fn validate_required_output_contract(
    ir: &IrModule,
    target_ir: &TargetIr,
    target: &str,
) -> Result<()> {
    if target != "cli" {
        return Ok(());
    }
    let Some(raw_outputs) = ir.meta.get("required_outputs") else {
        return Ok(());
    };
    let required_outputs = parse_meta_csv_list(raw_outputs);
    if required_outputs.is_empty() {
        return Ok(());
    }

    let Some(runtime_rules) = target_ir
        .extensions
        .get("runtimeRules")
        .and_then(|v| v.as_array())
    else {
        bail!(
            "C910: required_outputs configured but target IR has no extensions.runtimeRules for writer validation"
        );
    };

    let state_strings = target_ir
        .state
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<String, String>>()
        })
        .unwrap_or_default();

    let mut writer_paths: Vec<(String, String)> = Vec::new();
    for rule in runtime_rules {
        let Some(assigns) = rule.get("assign").and_then(|v| v.as_array()) else {
            continue;
        };
        for assign in assigns {
            let Some(call) = assign.pointer("/value/call") else {
                continue;
            };
            let Some(name) = call.get("name").and_then(|v| v.as_str()) else {
                continue;
            };
            if name != "writeJson" && name != "writeCsv" {
                continue;
            }
            let path_value = call.pointer("/args/0/value");
            let Some(output_path) = extract_runtime_path(path_value, &state_strings) else {
                continue;
            };
            writer_paths.push((name.to_string(), output_path));
        }
    }

    for required in required_outputs {
        let req_norm = normalize_path_like(&required);
        let needed_writer = if req_norm.ends_with(".json") {
            "writeJson"
        } else if req_norm.ends_with(".csv") {
            "writeCsv"
        } else {
            bail!(
                "C910: required_outputs entry '{}' has unsupported extension (expected .json or .csv)",
                required
            );
        };

        let matched = writer_paths
            .iter()
            .any(|(writer, p)| writer == needed_writer && path_like_match(p, &req_norm));
        if !matched {
            bail!(
                "C911: required output '{}' is not backed by deterministic '{}' call in runtime rules",
                required,
                needed_writer
            );
        }
    }

    Ok(())
}

fn extract_runtime_path(
    path_value: Option<&Value>,
    state_strings: &HashMap<String, String>,
) -> Option<String> {
    let v = path_value?;
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    if let Some(id) = v.get("ident").and_then(|x| x.as_str()) {
        if let Some(resolved) = state_strings.get(id) {
            return Some(resolved.clone());
        }
        return Some(id.to_string());
    }
    None
}

fn normalize_path_like(input: &str) -> String {
    input.replace('\\', "/")
}

fn path_like_match(actual: &str, expected: &str) -> bool {
    let a = normalize_path_like(actual);
    let e = normalize_path_like(expected);
    a == e || a.ends_with(&format!("/{e}"))
}

fn verify_required_outputs(ir: &IrModule) -> Result<()> {
    let raw = match ir.meta.get("required_outputs") {
        Some(v) => v,
        None => return Ok(()),
    };

    let outputs = parse_meta_csv_list(raw);
    if outputs.is_empty() {
        return Ok(());
    }

    let mut missing = Vec::new();
    for item in outputs {
        let p = PathBuf::from(&item);
        if !p.exists() {
            missing.push(format!("missing '{}'", p.display()));
            continue;
        }
        if let Ok(meta) = fs::metadata(&p) {
            if meta.is_file() && meta.len() == 0 {
                missing.push(format!("empty '{}'", p.display()));
            }
        }
    }

    if !missing.is_empty() {
        bail!(
            "Required outputs check failed (@meta required_outputs): {}",
            missing.join(", ")
        );
    }
    Ok(())
}

fn parse_meta_csv_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().trim_matches('"'))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[derive(Clone, Copy)]
struct CleanRetention {
    max_age_days: Option<u64>,
    keep_latest: Option<usize>,
    max_size_mb: Option<u64>,
}

fn clean_cmd(input: Option<&Path>, all: bool, retention: CleanRetention) -> Result<()> {
    let root = std::env::current_dir()?;
    validate_retention(retention)?;
    clean_impl(&root, input, all, retention)
}

fn clean_impl(
    root: &Path,
    input: Option<&Path>,
    all: bool,
    retention: CleanRetention,
) -> Result<()> {
    let has_retention = retention.max_age_days.is_some()
        || retention.keep_latest.is_some()
        || retention.max_size_mb.is_some();

    if all {
        if has_retention {
            bail!("--all cannot be combined with retention options");
        }
        let dist = root.join("dist");
        if dist.exists() {
            fs::remove_dir_all(&dist)?;
            println!("Removed {}", dist.display());
        } else {
            println!("Nothing to clean.");
        }
        return Ok(());
    }

    if has_retention {
        if input.is_some() {
            bail!("Retention options apply to dist root only; omit input");
        }
        return clean_retention(root, retention);
    }

    let Some(input) = input else {
        bail!("Provide an input file, use --all, or set retention options");
    };
    let dist_dir = root.join(dist_dir(input));
    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir)?;
        println!("Removed {}", dist_dir.display());
    } else {
        println!("Nothing to clean for {}", input.display());
    }
    Ok(())
}

#[derive(Clone)]
struct DistEntry {
    path: PathBuf,
    modified_ms: u128,
    size_bytes: u128,
}

struct CleanStats {
    removed_entries: usize,
    removed_bytes: u128,
    remaining_bytes: u128,
}

fn clean_retention(root: &Path, retention: CleanRetention) -> Result<()> {
    let stats = clean_retention_quiet(root, retention, None)?;
    println!("{}", format_clean_stats("Retention clean complete", &stats));
    Ok(())
}

fn clean_retention_quiet(
    root: &Path,
    retention: CleanRetention,
    protected_entry: Option<&Path>,
) -> Result<CleanStats> {
    let dist = root.join("dist");
    let protected_abs = protected_entry.map(|p| {
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            root.join(p)
        }
    });
    if !dist.exists() {
        return Ok(CleanStats {
            removed_entries: 0,
            removed_bytes: 0,
            remaining_bytes: 0,
        });
    }

    let mut entries = dist_entries(&dist)?;
    if entries.is_empty() {
        return Ok(CleanStats {
            removed_entries: 0,
            removed_bytes: 0,
            remaining_bytes: 0,
        });
    }

    let now = now_unix_ms();
    let mut removed = 0usize;
    let mut removed_bytes = 0u128;

    if let Some(days) = retention.max_age_days {
        let max_age_ms = (days as u128) * 24 * 60 * 60 * 1000;
        let cutoff = now.saturating_sub(max_age_ms);
        let mut keep = Vec::new();
        for e in entries {
            if e.modified_ms < cutoff && Some(&e.path) != protected_abs.as_ref() {
                removed_bytes = removed_bytes.saturating_add(e.size_bytes);
                remove_entry(&e.path)?;
                removed += 1;
            } else {
                keep.push(e);
            }
        }
        entries = keep;
    }

    entries.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
    if let Some(keep_n) = retention.keep_latest {
        let mut keep = Vec::new();
        for (idx, e) in entries.into_iter().enumerate() {
            if idx < keep_n || Some(&e.path) == protected_abs.as_ref() {
                keep.push(e);
            } else {
                removed_bytes = removed_bytes.saturating_add(e.size_bytes);
                remove_entry(&e.path)?;
                removed += 1;
            }
        }
        entries = keep;
    }

    if let Some(max_mb) = retention.max_size_mb {
        let budget = (max_mb as u128) * 1024 * 1024;
        entries.sort_by(|a, b| b.modified_ms.cmp(&a.modified_ms));
        let mut total: u128 = entries.iter().map(|e| e.size_bytes).sum();
        let mut idx = entries.len();
        while total > budget && idx > 0 {
            idx -= 1;
            let e = &entries[idx];
            if Some(&e.path) == protected_abs.as_ref() {
                continue;
            }
            removed_bytes = removed_bytes.saturating_add(e.size_bytes);
            total = total.saturating_sub(e.size_bytes);
            remove_entry(&e.path)?;
            removed += 1;
        }
    }

    let remaining = dist_entries(&dist)?;
    let remaining_bytes: u128 = remaining.iter().map(|e| e.size_bytes).sum();
    Ok(CleanStats {
        removed_entries: removed,
        removed_bytes,
        remaining_bytes,
    })
}

fn format_clean_stats(prefix: &str, stats: &CleanStats) -> String {
    format!(
        "{}: removed {} entries ({:.2} MB), remaining {:.2} MB",
        prefix,
        stats.removed_entries,
        stats.removed_bytes as f64 / (1024.0 * 1024.0),
        stats.remaining_bytes as f64 / (1024.0 * 1024.0)
    )
}

fn auto_clean_retention_from_config(cfg: &Config) -> Option<CleanRetention> {
    let clean = cfg.clean.as_ref()?;
    if clean.auto != Some(true) {
        return None;
    }
    let retention = CleanRetention {
        max_age_days: clean.max_age_days,
        keep_latest: clean.keep_latest,
        max_size_mb: clean.max_size_mb,
    };
    if retention.max_age_days.is_none()
        && retention.keep_latest.is_none()
        && retention.max_size_mb.is_none()
    {
        return None;
    }
    if validate_retention(retention).is_err() {
        return None;
    }
    Some(retention)
}

fn maybe_auto_clean_dist(active_dist_dir: &Path) {
    let cfg = load_config();
    let Some(retention) = auto_clean_retention_from_config(&cfg) else {
        return;
    };
    let root = match std::env::current_dir() {
        Ok(root) => root,
        Err(e) => {
            eprintln!("{} {}", style_accent("Auto-clean warning:"), e);
            return;
        }
    };
    match clean_retention_quiet(&root, retention, Some(active_dist_dir)) {
        Ok(stats) => {
            if stats.removed_entries > 0 {
                println!("{}", style_dim(&format_clean_stats("Auto-clean", &stats)));
            }
        }
        Err(e) => {
            eprintln!("{} {}", style_accent("Auto-clean warning:"), e);
        }
    }
}

fn validate_retention(retention: CleanRetention) -> Result<()> {
    if let Some(v) = retention.max_age_days {
        if v == 0 {
            bail!("--max-age-days must be >= 1");
        }
    }
    if let Some(v) = retention.keep_latest {
        if v == 0 {
            bail!("--keep-latest must be >= 1");
        }
    }
    if let Some(v) = retention.max_size_mb {
        if v == 0 {
            bail!("--max-size-mb must be >= 1");
        }
    }
    Ok(())
}

fn remove_entry(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn dist_entries(dist_root: &Path) -> Result<Vec<DistEntry>> {
    let mut out = Vec::new();
    for de in fs::read_dir(dist_root)? {
        let de = de?;
        let path = de.path();
        let meta = fs::metadata(&path)?;
        let modified_ms = modified_unix_ms(&meta);
        let size_bytes = entry_size_bytes(&path)?;
        out.push(DistEntry {
            path,
            modified_ms,
            size_bytes,
        });
    }
    Ok(out)
}

fn modified_unix_ms(meta: &fs::Metadata) -> u128 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn entry_size_bytes(path: &Path) -> Result<u128> {
    let meta = fs::metadata(path)?;
    if meta.is_file() {
        return Ok(meta.len() as u128);
    }
    let mut total = 0u128;
    for de in fs::read_dir(path)? {
        let de = de?;
        total = total.saturating_add(entry_size_bytes(&de.path())?);
    }
    Ok(total)
}

fn enforce_meta(ir: &IrModule, target: &str) -> Result<bool> {
    if let Some(t) = ir.meta.get("target") {
        if t.to_lowercase() != target.to_lowercase() {
            bail!(
                "Target mismatch: meta target is {}, but build target is {}",
                t,
                target
            );
        }
    }
    let layout_required = ir
        .meta
        .get("layout")
        .map(|v| v.to_lowercase() == "explicit")
        .unwrap_or(false);
    if layout_required && target != "gui" {
        bail!("layout=explicit is only valid for gui target");
    }
    Ok(layout_required)
}

fn resolve_target_from_meta(target: Option<&str>, ir: &IrModule) -> Result<String> {
    if let Some(t) = target {
        return Ok(t.to_string());
    }
    if let Some(meta) = ir.meta.get("target") {
        return Ok(meta.to_string());
    }
    bail!("Target required. Use --target or set @meta target=...")
}

struct ProviderInfo {
    name: String,
    model: String,
}

fn print_unified_header(action: &str, target: &str, input: &Path, provider: Option<&ProviderInfo>) {
    println!();
    let title = style_title("SCULPT");
    let rest = style_title(&format!("Compiler {}", env!("CARGO_PKG_VERSION")));
    let copyright = style_dim("(C) 2026 byte5 GmbH");
    println!("{title} {rest} - {copyright}");
    println!("{} {}", style_dim("Language:"), style_dim(LANGUAGE_DEFAULT));
    let action_s = style_accent(action);
    println!("{} {}", style_dim("Action:"), action_s);
    println!("{} {}", style_dim("Target:"), style_dim(target));
    println!(
        "{} {}",
        style_dim("Input: "),
        style_dim(&input.display().to_string())
    );
    if let Some(p) = provider {
        println!("{} {}", style_dim("Provider:"), style_dim(&p.name));
        println!("{} {}", style_dim("Model:   "), style_dim(&p.model));
    }
    println!("{}", style_divider());
}

fn print_unified_footer(artifacts: &[&str], token_usage: Option<&TokenUsage>) {
    println!();
    println!("{}", style_accent("Artifacts"));
    for a in artifacts {
        println!("  {}", style_dim(a));
    }
    println!();
    println!("{}", style_accent("Tokens"));
    if let Some(tokens) = token_usage {
        let input = tokens
            .input_tokens
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let output = tokens
            .output_tokens
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let total = tokens
            .total_tokens
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        println!(
            "  {}",
            style_dim(&format!(
                "input={} output={} total={}",
                input, output, total
            ))
        );
    } else {
        println!("  {}", style_dim("unavailable"));
    }
    println!("{}", style_divider());
}

fn style_title(s: &str) -> String {
    color_24(s, 0, 255, 255, true)
} // byte5 cyan
fn style_accent(s: &str) -> String {
    color_24(s, 234, 81, 114, true)
} // byte5 pink
fn style_dim(s: &str) -> String {
    color_24(s, 150, 160, 170, false)
}
fn style_divider() -> String {
    style_dim("────────────────────────────────────────────────────")
}
fn style_step(idx: &str, label: &str, status: &str) -> String {
    let bar = style_progress_bar(idx, status, 16);
    format!(
        "  {} {} {} {}",
        style_dim(&format!("{idx}.")),
        bar,
        label,
        style_accent(status)
    )
}

fn style_progress_bar(_idx: &str, status: &str, width: usize) -> String {
    let filled = if status == "ok" || status == "failed" {
        width
    } else if status.starts_with("running") {
        width / 2
    } else {
        0
    };
    let running = status.starts_with("running");

    let mut left = String::new();
    let mut right = String::new();
    for i in 0..width {
        if i < filled {
            left.push('█');
        } else {
            right.push('░');
        }
    }

    let mut bar = String::new();
    bar.push_str(&style_dim("["));
    bar.push_str(&color_24(&left, 0, 255, 255, true));
    if running && filled < width {
        bar.push_str(&color_24("▌", 234, 81, 114, true));
        let tail: String = right.chars().skip(1).collect();
        if !tail.is_empty() {
            bar.push_str(&style_dim(&tail));
        }
    } else {
        bar.push_str(&style_dim(&right));
    }
    bar.push_str(&style_dim("]"));
    bar.push(' ');
    let percent = if width == 0 {
        0
    } else {
        (filled * 100) / width
    };
    bar.push_str(&style_dim(&format!("{:>3}%", percent)));
    bar
}

fn print_step(idx: &str, label: &str, status: &str) {
    println!("{}", style_step(idx, label, status));
    flush_now();
}

fn finish_step(idx: &str, label: &str, status: &str) {
    replace_last_line(&style_step(idx, label, status));
}

fn flush_now() {
    let _ = io::stdout().flush();
}

fn replace_last_line(line: &str) {
    print!("\x1b[1A\r\x1b[2K{}\n", line);
    flush_now();
}

fn update_last_line(line: &str) {
    print!("\x1b[1A\r\x1b[2K{}\x1b[1B", line);
    flush_now();
}

struct SpinnerHandle {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

fn start_spinner(idx: &str, label: &str) -> SpinnerHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = stop.clone();
    let idx_s = idx.to_string();
    let label_s = label.to_string();
    let frames = ["|", "/", "-", "\\"];
    print_step(&idx_s, &label_s, &format!("running {}", frames[0]));
    let handle = thread::spawn(move || {
        let mut i = 0usize;
        while !stop_thread.load(Ordering::Relaxed) {
            let status = format!("running {}", frames[i % frames.len()]);
            update_last_line(&style_step(&idx_s, &label_s, &status));
            i = i.wrapping_add(1);
            thread::sleep(Duration::from_millis(120));
        }
    });
    SpinnerHandle {
        stop,
        handle: Some(handle),
    }
}

fn stop_spinner(mut spinner: SpinnerHandle) {
    spinner.stop.store(true, Ordering::Relaxed);
    if let Some(handle) = spinner.handle.take() {
        let _ = handle.join();
    }
}

fn color_24(s: &str, r: u8, g: u8, b: u8, bold: bool) -> String {
    if bold {
        format!("\x1b[1;38;2;{r};{g};{b}m{s}\x1b[0m")
    } else {
        format!("\x1b[38;2;{r};{g};{b}m{s}\x1b[0m")
    }
}

fn target_list() -> Result<()> {
    let targets = list_targets()?;
    println!("Language: default {}", LANGUAGE_DEFAULT);
    println!("Available targets:");
    for t in targets {
        println!("  {}", t);
    }
    Ok(())
}

fn gate_check(gate_file: &Path) -> Result<()> {
    let raw = fs::read_to_string(gate_file)
        .with_context(|| format!("Failed to read gate file {}", gate_file.display()))?;
    let raw_json: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid gate JSON in {}", gate_file.display()))?;
    if raw_json
        .get("criteria")
        .and_then(|c| c.as_array())
        .is_some()
    {
        let spec: GateSpec = serde_json::from_value(raw_json)
            .with_context(|| format!("Invalid classic gate JSON in {}", gate_file.display()))?;
        return gate_check_classic(gate_file, &spec);
    }

    let spec: DataHeavyGateSpec = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid data-heavy gate JSON in {}", gate_file.display()))?;
    gate_check_data_heavy(gate_file, &spec)
}

fn gate_check_classic(gate_file: &Path, spec: &GateSpec) -> Result<()> {
    println!();
    println!(
        "{} {}",
        style_title("SCULPT"),
        style_title(&format!("Gate Check {}", env!("CARGO_PKG_VERSION")))
    );
    println!("{} {}", style_dim("Gate: "), style_dim(&spec.name));
    println!(
        "{} {}",
        style_dim("File: "),
        style_dim(&gate_file.display().to_string())
    );
    if let Some(study) = &spec.study {
        println!("{} {}", style_dim("Study:"), style_dim(study));
    }
    println!("{}", style_divider());

    let mut failed = 0usize;
    for c in &spec.criteria {
        let result = evaluate_gate_criterion(c)?;
        let status = if result.passed {
            style_title("PASS")
        } else {
            style_accent("FAIL")
        };
        println!(
            "{} {} sculpt={:.3} vibe={:.3} op={} {}",
            style_dim(&format!("[{}]", c.id)),
            style_dim(&c.description),
            c.sculpt,
            c.vibe,
            c.operator,
            status
        );
        if let Some(delta) = c.min_delta {
            println!("  {} {:.3}", style_dim("min_delta:"), delta);
        }
        println!("  {} {}", style_dim("detail:"), style_dim(&result.detail));
        if !result.passed {
            failed += 1;
        }
    }

    println!("{}", style_divider());
    if failed == 0 {
        println!("{} {}", style_title("Gate Result:"), style_title("PASS"));
        Ok(())
    } else {
        bail!("Gate Result: FAIL ({} criteria failed)", failed)
    }
}

fn gate_check_data_heavy(gate_file: &Path, spec: &DataHeavyGateSpec) -> Result<()> {
    let observed = spec
        .observed
        .as_ref()
        .or(spec.criteria.as_ref())
        .ok_or_else(|| anyhow::anyhow!("Missing observed/criteria object in data-heavy gate"))?;

    println!();
    println!(
        "{} {}",
        style_title("SCULPT"),
        style_title(&format!("Gate Check {}", env!("CARGO_PKG_VERSION")))
    );
    println!("{} {}", style_dim("Gate: "), style_dim(&spec.name));
    println!(
        "{} {}",
        style_dim("File: "),
        style_dim(&gate_file.display().to_string())
    );
    println!(
        "{} {}",
        style_dim("Metrics:"),
        style_dim(&spec.source_metrics)
    );
    println!("{}", style_divider());

    if observed.infra_blocked {
        println!(
            "{} {}",
            style_accent("Gate Result:"),
            style_accent("INFRA BLOCKED (provider availability/quota)")
        );
        bail!(
            "Gate Result: INFRA BLOCKED (infra_failures={})",
            observed.infra_failures
        );
    }

    let mut failed = 0usize;
    let checks = vec![
        (
            "G1",
            "Acceptance rate",
            observed.acceptance_rate >= spec.thresholds.min_acceptance_rate,
            format!(
                "observed {:.3}, required >= {:.3}",
                observed.acceptance_rate, spec.thresholds.min_acceptance_rate
            ),
        ),
        (
            "G2",
            "Repro pass count",
            observed.repro_pass >= spec.thresholds.min_repro_pass,
            format!(
                "observed {}, required >= {}",
                observed.repro_pass, spec.thresholds.min_repro_pass
            ),
        ),
        (
            "G3",
            "Repro unique hashes",
            observed.repro_unique_hashes <= spec.thresholds.max_repro_unique_hashes,
            format!(
                "observed {}, required <= {}",
                observed.repro_unique_hashes, spec.thresholds.max_repro_unique_hashes
            ),
        ),
        (
            "G4",
            "Reproducibility flag",
            !spec.thresholds.require_reproducible || observed.reproducible,
            format!(
                "observed {}, required {}",
                observed.reproducible, spec.thresholds.require_reproducible
            ),
        ),
    ];

    for (id, desc, ok, detail) in checks {
        let status = if ok {
            style_title("PASS")
        } else {
            style_accent("FAIL")
        };
        println!(
            "{} {} {}",
            style_dim(&format!("[{}]", id)),
            style_dim(desc),
            status
        );
        println!("  {} {}", style_dim("detail:"), style_dim(&detail));
        if !ok {
            failed += 1;
        }
    }

    println!("{}", style_divider());
    if failed == 0 {
        println!("{} {}", style_title("Gate Result:"), style_title("PASS"));
        Ok(())
    } else {
        bail!("Gate Result: FAIL ({} criteria failed)", failed)
    }
}

#[derive(serde::Serialize)]
struct DataHeavyRunResult {
    size: String,
    build_rc: i32,
    run_rc: i32,
    build_s: f64,
    run_s: f64,
    ok: bool,
    report_exists: bool,
    exceptions_exists: bool,
    provider_used: String,
    model_used: Option<String>,
    fallback_used: bool,
    provider_attempts: Vec<serde_json::Value>,
    report_path: String,
    exceptions_path: String,
    normalized_hash: Option<String>,
    failure_kind: String,
    failure_reason: Option<String>,
    errors: Vec<String>,
}

#[derive(Debug, Clone)]
struct BenchmarkBuildAttempt {
    provider: Option<String>,
    model: Option<String>,
    ok: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct BenchmarkBuildResult {
    rc: i32,
    elapsed_s: f64,
    provider_used: String,
    model_used: Option<String>,
    fallback_used: bool,
    attempts: Vec<BenchmarkBuildAttempt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BenchmarkFailureKind {
    None,
    Infra,
    Product,
}

fn benchmark_data_heavy(
    script: &Path,
    dataset_root: &Path,
    sizes: &str,
    repro_runs: usize,
    output: &Path,
    gate_output: &Path,
    provider: Option<String>,
    model: Option<String>,
    strict_provider: bool,
    target: &str,
) -> Result<()> {
    if target != "cli" {
        bail!("benchmark data-heavy currently supports only --target cli");
    }

    let sizes: Vec<String> = sizes
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    if sizes.is_empty() {
        bail!("No benchmark sizes provided");
    }

    let script_text = fs::read_to_string(script)
        .with_context(|| format!("Failed to read benchmark script {}", script.display()))?;

    let tmp_dir = PathBuf::from("poc/tmp/data_heavy_benchmark");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }
    fs::create_dir_all(&tmp_dir)?;

    println!();
    println!(
        "{} {}",
        style_title("SCULPT"),
        style_title(&format!("Benchmark {}", env!("CARGO_PKG_VERSION")))
    );
    println!("{} {}", style_dim("Type:"), style_dim("data-heavy"));
    println!(
        "{} {}",
        style_dim("Script:"),
        style_dim(&script.display().to_string())
    );
    println!("{}", style_divider());

    let provider_chain = benchmark_provider_chain(provider.clone(), strict_provider);
    let mut runs: Vec<DataHeavyRunResult> = Vec::new();
    for size in &sizes {
        let ds_dir = dataset_root.join(size);
        let invoices = ds_dir.join("invoices.csv");
        let payments = ds_dir.join("payments.csv");
        if !invoices.exists() || !payments.exists() {
            bail!(
                "Missing dataset files for size '{}': {} / {}",
                size,
                invoices.display(),
                payments.display()
            );
        }

        let run_dir = PathBuf::from("poc/runs/data_heavy_sculpt").join(size);
        fs::create_dir_all(&run_dir)?;
        let report_path = run_dir.join("reconciliation_report.json");
        let exceptions_path = run_dir.join("exceptions.csv");
        let _ = fs::remove_file(&report_path);
        let _ = fs::remove_file(&exceptions_path);

        let variant_script = tmp_dir.join(format!("invoice_reconciliation_batch_{}.sculpt", size));
        let variant_source = with_data_paths(
            &script_text,
            &invoices,
            &payments,
            &report_path,
            &exceptions_path,
        )?;
        fs::write(&variant_script, variant_source)?;

        let build_result = benchmark_build_with_fallback(
            &variant_script,
            target,
            &provider_chain,
            model.clone(),
            strict_provider,
        );
        let build_rc = build_result.rc;
        let build_s = build_result.elapsed_s;

        let mut run_rc = 1;
        let mut run_s = 0.0f64;
        if build_rc == 0 {
            let run_start = Instant::now();
            run_rc =
                run_cli_noninteractive(&dist_dir(&variant_script), &report_path, &exceptions_path)
                    .unwrap_or(1);
            run_s = run_start.elapsed().as_secs_f64();
        }
        let mut errors: Vec<String> = Vec::new();
        for attempt in &build_result.attempts {
            if let Some(err) = &attempt.error {
                errors.push(format!(
                    "provider {} failed: {}",
                    attempt.provider.as_deref().unwrap_or("default"),
                    err
                ));
            }
        }

        let (ok, normalized_hash, mut validation_errors) =
            validate_data_heavy_outputs(&report_path, &exceptions_path);
        errors.append(&mut validation_errors);
        let (failure_kind, failure_reason) =
            classify_data_heavy_failure(build_rc, run_rc, &build_result.attempts, &errors);
        runs.push(DataHeavyRunResult {
            size: size.clone(),
            build_rc,
            run_rc,
            build_s,
            run_s,
            ok: build_rc == 0 && run_rc == 0 && ok,
            report_exists: report_path.exists(),
            exceptions_exists: exceptions_path.exists(),
            provider_used: build_result.provider_used,
            model_used: build_result.model_used,
            fallback_used: build_result.fallback_used,
            provider_attempts: build_result
                .attempts
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "provider": a.provider.as_deref().unwrap_or("default"),
                        "model": a.model,
                        "ok": a.ok,
                        "error": a.error,
                    })
                })
                .collect(),
            report_path: report_path.display().to_string(),
            exceptions_path: exceptions_path.display().to_string(),
            normalized_hash,
            failure_kind: match failure_kind {
                BenchmarkFailureKind::None => "none".to_string(),
                BenchmarkFailureKind::Infra => "infra".to_string(),
                BenchmarkFailureKind::Product => "product".to_string(),
            },
            failure_reason,
            errors,
        });
    }

    let repro_size = if sizes.iter().any(|s| s == "medium") {
        "medium".to_string()
    } else {
        sizes[0].clone()
    };
    let mut repro: Vec<serde_json::Value> = Vec::new();
    for i in 0..repro_runs {
        let ds_dir = dataset_root.join(&repro_size);
        let invoices = ds_dir.join("invoices.csv");
        let payments = ds_dir.join("payments.csv");
        let run_dir = PathBuf::from("poc/runs/data_heavy_sculpt")
            .join("repro")
            .join(format!("run_{}", i + 1));
        fs::create_dir_all(&run_dir)?;
        let report_path = run_dir.join("reconciliation_report.json");
        let exceptions_path = run_dir.join("exceptions.csv");
        let _ = fs::remove_file(&report_path);
        let _ = fs::remove_file(&exceptions_path);

        let variant_script = tmp_dir.join(format!(
            "invoice_reconciliation_batch_repro_{}.sculpt",
            i + 1
        ));
        let variant_source = with_data_paths(
            &script_text,
            &invoices,
            &payments,
            &report_path,
            &exceptions_path,
        )?;
        fs::write(&variant_script, variant_source)?;

        let build_result = benchmark_build_with_fallback(
            &variant_script,
            target,
            &provider_chain,
            model.clone(),
            strict_provider,
        );
        let build_rc = build_result.rc;
        let build_s = build_result.elapsed_s;

        let run_start = Instant::now();
        let run_rc = if build_rc == 0 {
            run_cli_noninteractive(&dist_dir(&variant_script), &report_path, &exceptions_path)
                .unwrap_or(1)
        } else {
            1
        };
        let run_s = run_start.elapsed().as_secs_f64();

        let mut errors: Vec<String> = Vec::new();
        for attempt in &build_result.attempts {
            if let Some(err) = &attempt.error {
                errors.push(format!(
                    "provider {} failed: {}",
                    attempt.provider.as_deref().unwrap_or("default"),
                    err
                ));
            }
        }
        let (ok, normalized_hash, mut validation_errors) =
            validate_data_heavy_outputs(&report_path, &exceptions_path);
        errors.append(&mut validation_errors);
        let (failure_kind, failure_reason) =
            classify_data_heavy_failure(build_rc, run_rc, &build_result.attempts, &errors);
        repro.push(serde_json::json!({
            "run": i + 1,
            "size": repro_size,
            "build_rc": build_rc,
            "run_rc": run_rc,
            "build_s": build_s,
            "run_s": run_s,
            "ok": build_rc == 0 && run_rc == 0 && ok,
            "hash": normalized_hash,
            "failure_kind": match failure_kind {
                BenchmarkFailureKind::None => "none",
                BenchmarkFailureKind::Infra => "infra",
                BenchmarkFailureKind::Product => "product",
            },
            "failure_reason": failure_reason,
            "errors": errors,
            "provider_used": build_result.provider_used,
            "model_used": build_result.model_used,
            "fallback_used": build_result.fallback_used,
            "provider_attempts": build_result
                .attempts
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "provider": a.provider.as_deref().unwrap_or("default"),
                        "model": a.model,
                        "ok": a.ok,
                        "error": a.error
                    })
                })
                .collect::<Vec<_>>()
        }));
    }

    let accepted_runs = runs.iter().filter(|r| r.ok).count();
    let infra_failures = runs
        .iter()
        .filter(|r| r.failure_kind.as_str() == "infra")
        .count();
    let matrix_evaluable = runs.len().saturating_sub(infra_failures);
    let repro_ok: Vec<&serde_json::Value> = repro
        .iter()
        .filter(|r| r.get("ok").and_then(|v| v.as_bool()) == Some(true))
        .collect();
    let mut unique_hashes = HashSet::new();
    for r in &repro_ok {
        if let Some(h) = r.get("hash").and_then(|v| v.as_str()) {
            unique_hashes.insert(h.to_string());
        }
    }
    let reproducible = !repro_ok.is_empty() && unique_hashes.len() == 1;

    let fallback_runs = runs.iter().filter(|r| r.fallback_used).count();
    let fallback_repro_runs = repro
        .iter()
        .filter(|r| r.get("fallback_used").and_then(|v| v.as_bool()) == Some(true))
        .count();
    let repro_infra_failures = repro
        .iter()
        .filter(|r| r.get("failure_kind").and_then(|v| v.as_str()) == Some("infra"))
        .count();
    let repro_evaluable = repro_runs.saturating_sub(repro_infra_failures);
    let infra_blocked = matrix_evaluable == 0 || repro_evaluable == 0;
    let acceptance_rate = if matrix_evaluable == 0 {
        0.0
    } else {
        accepted_runs as f64 / matrix_evaluable as f64
    };

    let metrics = serde_json::json!({
        "provider": provider.clone().unwrap_or_else(|| "stub".to_string()),
        "model": model,
        "provider_strategy": {
            "requested_provider": provider,
            "requested_model": model,
            "strict_provider": strict_provider,
            "fallback_chain": provider_chain
        },
        "target": target,
        "runs": runs,
        "repro": repro,
        "summary": {
            "matrix_total": sizes.len(),
            "matrix_evaluable": matrix_evaluable,
            "infra_failures": infra_failures,
            "matrix_pass": accepted_runs,
            "acceptance_rate": acceptance_rate,
            "repro_runs": repro_runs,
            "repro_evaluable": repro_evaluable,
            "repro_infra_failures": repro_infra_failures,
            "repro_pass": repro_ok.len(),
            "repro_unique_hashes": unique_hashes.len(),
            "reproducible": reproducible,
            "fallback_runs": fallback_runs,
            "fallback_repro_runs": fallback_repro_runs,
            "infra_blocked": infra_blocked
        }
    });

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        output,
        format!("{}\n", serde_json::to_string_pretty(&metrics)?),
    )?;

    let gate = serde_json::json!({
        "name": "data_heavy_sculpt_gate_input_v1",
        "source_metrics": output.display().to_string(),
        "thresholds": {
            "min_acceptance_rate": 1.0,
            "min_repro_pass": 5,
            "max_repro_unique_hashes": 1,
            "require_reproducible": true
        },
        "observed": {
            "acceptance_rate": metrics["summary"]["acceptance_rate"],
            "repro_pass": metrics["summary"]["repro_pass"],
            "repro_unique_hashes": metrics["summary"]["repro_unique_hashes"],
            "reproducible": metrics["summary"]["reproducible"],
            "infra_blocked": metrics["summary"]["infra_blocked"],
            "infra_failures": metrics["summary"]["infra_failures"]
        }
    });
    if let Some(parent) = gate_output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        gate_output,
        format!("{}\n", serde_json::to_string_pretty(&gate)?),
    )?;

    println!("{}", style_divider());
    println!(
        "{} {} / {}",
        style_dim("Matrix pass:"),
        accepted_runs,
        sizes.len()
    );
    println!(
        "{} {} (infra={} evaluable={})",
        style_dim("Acceptance rate:"),
        style_dim(&format!("{:.3}", acceptance_rate)),
        infra_failures,
        matrix_evaluable
    );
    println!(
        "{} {} / {}",
        style_dim("Repro pass:"),
        repro_ok.len(),
        repro_runs
    );
    println!(
        "{} {}",
        style_dim("Repro unique hashes:"),
        unique_hashes.len()
    );
    println!(
        "{} {} / {}",
        style_dim("Fallback runs:"),
        fallback_runs,
        sizes.len()
    );
    if infra_blocked {
        println!(
            "{} {}",
            style_accent("Benchmark status:"),
            style_accent("INFRA BLOCKED (provider availability/quota)")
        );
    }
    println!(
        "{} {}",
        style_dim("Metrics:"),
        style_dim(&output.display().to_string())
    );
    println!(
        "{} {}",
        style_dim("Gate input:"),
        style_dim(&gate_output.display().to_string())
    );
    Ok(())
}

fn benchmark_provider_chain(
    requested_provider: Option<String>,
    strict_provider: bool,
) -> Vec<String> {
    if strict_provider {
        return vec![requested_provider.unwrap_or_else(|| "stub".to_string())];
    }

    let requested = requested_provider.unwrap_or_else(|| "openai".to_string());
    let mut chain = Vec::new();
    chain.push(requested.clone());
    match requested.as_str() {
        "openai" => {
            chain.push("gemini".to_string());
            chain.push("stub".to_string());
        }
        "gemini" => {
            chain.push("openai".to_string());
            chain.push("stub".to_string());
        }
        "anthropic" => {
            chain.push("openai".to_string());
            chain.push("gemini".to_string());
            chain.push("stub".to_string());
        }
        "stub" => {}
        _ => {
            chain.push("stub".to_string());
        }
    }

    let mut seen = HashSet::new();
    chain
        .into_iter()
        .filter(|p| seen.insert(p.clone()))
        .collect()
}

fn benchmark_build_with_fallback(
    input: &Path,
    target: &str,
    provider_chain: &[String],
    model: Option<String>,
    strict_provider: bool,
) -> BenchmarkBuildResult {
    let started = Instant::now();
    let mut attempts = Vec::new();
    let mut provider_used = "unknown".to_string();
    let mut model_used = model.clone();
    let mut fallback_used = false;
    let mut last_error = None::<String>;

    for (idx, candidate) in provider_chain.iter().enumerate() {
        let candidate_model = if idx == 0 { model.clone() } else { None };
        match build(
            input,
            Some(target),
            None,
            Some(candidate.clone()),
            candidate_model.clone(),
            strict_provider,
            None,
        ) {
            Ok(()) => {
                attempts.push(BenchmarkBuildAttempt {
                    provider: Some(candidate.clone()),
                    model: candidate_model.clone(),
                    ok: true,
                    error: None,
                });
                provider_used = candidate.clone();
                model_used = candidate_model;
                fallback_used = idx > 0;
                return BenchmarkBuildResult {
                    rc: 0,
                    elapsed_s: started.elapsed().as_secs_f64(),
                    provider_used,
                    model_used,
                    fallback_used,
                    attempts,
                };
            }
            Err(e) => {
                let msg = e.to_string();
                attempts.push(BenchmarkBuildAttempt {
                    provider: Some(candidate.clone()),
                    model: candidate_model,
                    ok: false,
                    error: Some(msg.clone()),
                });
                last_error = Some(msg.clone());
                if strict_provider
                    || idx + 1 >= provider_chain.len()
                    || !is_provider_unavailable_error(&msg)
                {
                    break;
                }
                if let Some(next) = provider_chain.get(idx + 1) {
                    eprintln!(
                        "{} switching to fallback provider '{}'...",
                        style_dim("Benchmark provider fallback:"),
                        style_dim(next)
                    );
                }
            }
        }
    }

    if let Some(err) = last_error {
        eprintln!("{} {}", style_accent("Build failed:"), err);
    }

    BenchmarkBuildResult {
        rc: 1,
        elapsed_s: started.elapsed().as_secs_f64(),
        provider_used,
        model_used,
        fallback_used,
        attempts,
    }
}

fn classify_data_heavy_failure(
    build_rc: i32,
    run_rc: i32,
    attempts: &[BenchmarkBuildAttempt],
    errors: &[String],
) -> (BenchmarkFailureKind, Option<String>) {
    if build_rc == 0 && run_rc == 0 {
        return (BenchmarkFailureKind::None, None);
    }

    if build_rc != 0 {
        let has_attempts = !attempts.is_empty();
        let provider_only_failures = has_attempts
            && attempts
                .iter()
                .filter_map(|a| a.error.as_deref())
                .all(is_provider_unavailable_error);
        if provider_only_failures {
            let reason = attempts
                .iter()
                .filter_map(|a| a.error.as_ref())
                .next()
                .cloned();
            return (BenchmarkFailureKind::Infra, reason);
        }

        let reason = errors.first().cloned().or_else(|| {
            attempts
                .iter()
                .filter_map(|a| a.error.as_ref())
                .next()
                .cloned()
        });
        return (BenchmarkFailureKind::Product, reason);
    }

    let reason = errors.first().cloned();
    (BenchmarkFailureKind::Product, reason)
}

fn is_provider_unavailable_error(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    [
        "insufficient_quota",
        "status 429",
        "too many requests",
        "rate limit",
        "api key",
        "authentication",
        "unauthorized",
        "forbidden",
        "openai error",
        "gemini error",
        "anthropic error",
        "connection reset",
        "timed out",
        "strict-provider",
    ]
    .iter()
    .any(|needle| m.contains(needle))
}

fn with_data_paths(
    source: &str,
    invoices: &Path,
    payments: &Path,
    report: &Path,
    exceptions: &Path,
) -> Result<String> {
    let mut out = source.to_string();
    out = replace_string_assignment(&out, "invoicesPath", &invoices.display().to_string())?;
    out = replace_string_assignment(&out, "paymentsPath", &payments.display().to_string())?;
    out = replace_string_assignment(&out, "reportPath", &report.display().to_string())?;
    out = replace_string_assignment(&out, "exceptionsPath", &exceptions.display().to_string())?;
    Ok(out)
}

fn replace_string_assignment(source: &str, variable: &str, value: &str) -> Result<String> {
    let mut changed = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&format!("{variable} = \"")) {
            let indent_len = line.len().saturating_sub(trimmed.len());
            let indent = &line[..indent_len];
            let escaped = value.replace('\\', "/");
            lines.push(format!("{indent}{variable} = \"{escaped}\""));
            changed = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !changed {
        bail!(
            "Could not patch assignment '{} = \"...\"' in benchmark script",
            variable
        );
    }
    Ok(lines.join("\n") + "\n")
}

fn run_cli_noninteractive(
    dist_dir: &Path,
    report_path: &Path,
    exceptions_path: &Path,
) -> Result<i32> {
    let entry = dist_dir.join("main.js");
    if !entry.exists() {
        bail!("{} not found", entry.display());
    }
    let mut child = ProcCommand::new("node")
        .arg(&entry)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| {
            format!(
                "Failed to run benchmark cli target (node {})",
                entry.display()
            )
        })?;

    thread::sleep(Duration::from_millis(200));
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(b"\n");
    }

    let deadline = Instant::now() + Duration::from_secs(30);
    let mut produced = false;
    while Instant::now() < deadline {
        if report_path.exists() && exceptions_path.exists() {
            let report_ok = fs::metadata(report_path)
                .map(|m| m.len() > 2)
                .unwrap_or(false);
            let exceptions_ok = fs::metadata(exceptions_path)
                .map(|m| m.len() > 0)
                .unwrap_or(false);
            if report_ok && exceptions_ok {
                produced = true;
                break;
            }
        }
        if let Some(status) = child.try_wait()? {
            return Ok(status.code().unwrap_or(1));
        }
        thread::sleep(Duration::from_millis(100));
    }

    if produced {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(b"\x1b");
        }
        let wait_deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < wait_deadline {
            if let Some(status) = child.try_wait()? {
                return Ok(status.code().unwrap_or(0));
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    let _ = child.kill();
    let status = child.wait()?;
    Ok(status.code().unwrap_or(if produced { 0 } else { 1 }))
}

fn validate_data_heavy_outputs(
    report_path: &Path,
    exceptions_path: &Path,
) -> (bool, Option<String>, Vec<String>) {
    let mut errors = Vec::new();
    if !report_path.exists() {
        errors.push("missing report json".to_string());
        return (false, None, errors);
    }
    if !exceptions_path.exists() {
        errors.push("missing exceptions csv".to_string());
        return (false, None, errors);
    }

    let report_raw = match fs::read_to_string(report_path) {
        Ok(s) => s,
        Err(e) => {
            errors.push(format!("report read error: {e}"));
            return (false, None, errors);
        }
    };
    let mut report_val: serde_json::Value = match serde_json::from_str(&report_raw) {
        Ok(v) => v,
        Err(e) => {
            errors.push(format!("report parse error: {e}"));
            return (false, None, errors);
        }
    };
    let Some(root) = report_val.as_object() else {
        errors.push("report root is not an object".to_string());
        return (false, None, errors);
    };

    let required_counts = [
        "matched_full",
        "matched_partial",
        "overpaid",
        "missing_payment",
        "duplicate_payment",
        "ambiguous",
        "suspicious",
    ];
    if root
        .get("input_stats")
        .and_then(|v| v.as_object())
        .is_none()
    {
        errors.push("report.input_stats missing".to_string());
    }
    let counts = root
        .get("classification_counts")
        .and_then(|v| v.as_object());
    if counts.is_none() {
        errors.push("report.classification_counts missing".to_string());
    } else if let Some(c) = counts {
        for key in required_counts {
            if c.get(key).and_then(|v| v.as_f64()).is_none() {
                errors.push(format!("classification_counts.{key} missing or non-number"));
            }
        }
    }

    let csv_raw = match fs::read_to_string(exceptions_path) {
        Ok(s) => s,
        Err(e) => {
            errors.push(format!("exceptions read error: {e}"));
            return (false, None, errors);
        }
    };
    let mut lines = csv_raw.lines();
    let header = lines.next().unwrap_or_default();
    if header.trim() != "invoice_id,payment_id,classification,reason" {
        errors.push("exceptions header mismatch".to_string());
    }
    let mut prev: Option<(String, String)> = None;
    for line in lines {
        let mut parts = line.splitn(4, ',');
        let invoice_id = parts.next().unwrap_or_default().to_string();
        let payment_id = parts.next().unwrap_or_default().to_string();
        let current = (invoice_id, payment_id);
        if let Some(p) = &prev {
            if current < *p {
                errors.push("exceptions rows not sorted by invoice_id,payment_id".to_string());
                break;
            }
        }
        prev = Some(current);
    }

    if let Some(obj) = report_val.as_object_mut() {
        obj.remove("generated_at");
        obj.remove("processing_ms");
    }
    let canonical = serde_json::to_string(&report_val).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hasher.update(b"\n");
    hasher.update(csv_raw.as_bytes());
    let normalized_hash = Some(hex::encode(hasher.finalize()));

    (errors.is_empty(), normalized_hash, errors)
}

struct GateEvalResult {
    passed: bool,
    detail: String,
}

fn evaluate_gate_criterion(c: &GateCriterion) -> Result<GateEvalResult> {
    let min_delta = c.min_delta.unwrap_or(0.0);
    let (passed, detail) = match c.operator.as_str() {
        "sculpt_gt_vibe" => {
            let delta = c.sculpt - c.vibe;
            let ok = c.sculpt > c.vibe && delta >= min_delta;
            (
                ok,
                format!(
                    "delta={:.3}, requires sculpt>vibe and delta>={:.3}",
                    delta, min_delta
                ),
            )
        }
        "sculpt_gte_vibe" => {
            let delta = c.sculpt - c.vibe;
            let ok = c.sculpt >= c.vibe && delta >= min_delta;
            (
                ok,
                format!(
                    "delta={:.3}, requires sculpt>=vibe and delta>={:.3}",
                    delta, min_delta
                ),
            )
        }
        "sculpt_lt_vibe" => {
            let delta = c.vibe - c.sculpt;
            let ok = c.sculpt < c.vibe && delta >= min_delta;
            (
                ok,
                format!(
                    "delta={:.3}, requires sculpt<vibe and delta>={:.3}",
                    delta, min_delta
                ),
            )
        }
        "sculpt_lte_vibe" => {
            let delta = c.vibe - c.sculpt;
            let ok = c.sculpt <= c.vibe && delta >= min_delta;
            (
                ok,
                format!(
                    "delta={:.3}, requires sculpt<=vibe and delta>={:.3}",
                    delta, min_delta
                ),
            )
        }
        "equal" => {
            let tolerance = c.min_delta.unwrap_or(0.0);
            let diff = (c.sculpt - c.vibe).abs();
            let ok = diff <= tolerance;
            (
                ok,
                format!("abs_diff={:.3}, requires abs_diff<={:.3}", diff, tolerance),
            )
        }
        other => bail!(
            "Unsupported gate operator '{}' in criterion '{}'",
            other,
            c.id
        ),
    };
    Ok(GateEvalResult { passed, detail })
}

fn target_describe(target: &str) -> Result<()> {
    let spec = describe_target(target)?;
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}

fn target_packages(target: &str) -> Result<()> {
    let spec = describe_target(target)?;
    let packages = spec
        .pointer("/contract/packages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if packages.is_empty() {
        println!("No packages declared for target '{}'", target);
        return Ok(());
    }
    println!("Packages for target '{}':", target);
    for pkg in packages {
        let id = pkg.get("id").and_then(Value::as_str).unwrap_or("<unknown>");
        let ns = pkg
            .get("namespace")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let desc = pkg.get("description").and_then(Value::as_str).unwrap_or("");
        println!("  {}  namespace={}  {}", id, ns, desc);
    }
    Ok(())
}

fn target_exports(target: &str, package: &str) -> Result<()> {
    let spec = describe_target(target)?;
    let packages = spec
        .pointer("/contract/packages")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let Some(pkg) = packages.into_iter().find(|p| {
        p.get("id")
            .and_then(Value::as_str)
            .map(|id| id == package)
            .unwrap_or(false)
    }) else {
        bail!("Unknown package '{}' for target '{}'", package, target);
    };
    let ns = pkg
        .get("namespace")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>");
    println!("Exports for package '{}' (namespace={}):", package, ns);
    if let Some(exports) = pkg.get("exports").and_then(Value::as_array) {
        for symbol in exports {
            if let Some(s) = symbol.as_str() {
                println!("  {}.{}", ns, s);
            }
        }
    } else {
        println!("  <none>");
    }
    Ok(())
}

fn target_stacks(target: &str) -> Result<()> {
    let spec = describe_target(target)?;
    let adapters = spec
        .pointer("/support/adapters")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if adapters.is_empty() {
        println!("No stack adapters declared for target '{}'", target);
        return Ok(());
    }
    println!("Stack adapters for target '{}':", target);
    for adapter in adapters {
        let id = adapter
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("<unknown>");
        let class = adapter
            .get("class")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let desc = adapter
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("");
        println!("  {}  class={}  {}", id, class, desc);
    }
    Ok(())
}

fn auth_check(provider: &str, verify: bool) -> Result<()> {
    match provider {
        "openai" => {
            let config = load_config();
            let key = env::var("OPENAI_API_KEY")
                .ok()
                .or_else(|| config.openai.and_then(|c| c.api_key));
            let Some(api_key) = key else {
                bail!("OPENAI_API_KEY not set and no key in sculpt.config.json");
            };

            if !verify {
                println!("OpenAI provider: API key found");
                return Ok(());
            }

            let client = reqwest::blocking::Client::new();
            let resp = client
                .get("https://api.openai.com/v1/models")
                .bearer_auth(api_key)
                .send()?;
            if !resp.status().is_success() {
                bail!("OpenAI auth check failed: status {}", resp.status());
            }
            println!("OpenAI provider: API key verified");
            Ok(())
        }
        "anthropic" => {
            let config = load_config();
            let key = env::var("ANTHROPIC_API_KEY")
                .ok()
                .or_else(|| config.anthropic.as_ref().and_then(|c| c.api_key.clone()));
            let Some(api_key) = key else {
                bail!("ANTHROPIC_API_KEY not set and no key in sculpt.config.json");
            };

            if !verify {
                println!("Anthropic provider: API key found");
                return Ok(());
            }

            let model = config
                .anthropic
                .as_ref()
                .and_then(|c| c.model.clone())
                .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
            let client = reqwest::blocking::Client::new();
            let body = serde_json::json!({
              "model": model,
              "max_tokens": 1,
              "system": "ping",
              "messages": [{ "role": "user", "content": "ping" }]
            });
            let resp = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()?;
            if !resp.status().is_success() {
                bail!("Anthropic auth check failed: status {}", resp.status());
            }
            println!("Anthropic provider: API key verified");
            Ok(())
        }
        "gemini" => {
            let config = load_config();
            let key = env::var("GEMINI_API_KEY")
                .ok()
                .or_else(|| config.gemini.as_ref().and_then(|c| c.api_key.clone()));
            let Some(api_key) = key else {
                bail!("GEMINI_API_KEY not set and no key in sculpt.config.json");
            };

            if !verify {
                println!("Gemini provider: API key found");
                return Ok(());
            }

            let model = config
                .gemini
                .as_ref()
                .and_then(|c| c.model.clone())
                .unwrap_or_else(|| "gemini-2.5-pro".to_string());
            let body = serde_json::json!({
              "contents": [{ "role": "user", "parts": [{ "text": "ping" }] }],
              "generationConfig": { "maxOutputTokens": 1 }
            });
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                model
            );
            let client = reqwest::blocking::Client::new();
            let resp = client
                .post(url)
                .header("x-goog-api-key", api_key)
                .header("content-type", "application/json")
                .json(&body)
                .send()?;
            if !resp.status().is_success() {
                bail!("Gemini auth check failed: status {}", resp.status());
            }
            println!("Gemini provider: API key verified");
            Ok(())
        }
        other => bail!("Unknown provider: {}", other),
    }
}

fn load_ir(input: &Path, nd_policy_override: Option<&str>) -> Result<crate::ir::IrModule> {
    let (mut module, imported_roots) = if is_project_file(input) {
        let project = load_project_context(input)?;
        let (_, entry_module) = project
            .modules
            .get(&project.entry_module)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!("Project entry module '{}' not found", project.entry_module)
            })?;
        let roots = resolve_imported_module_roots_project(&entry_module, &project.modules)?;
        (entry_module, roots)
    } else {
        let src =
            fs::read_to_string(input).with_context(|| format!("Failed to read {:?}", input))?;
        let module = parse_source(&src)?;
        if !module.imports.is_empty() {
            bail!(
                "Imports require a project file (*.sculpt.json). Stand-alone scripts cannot import modules."
            );
        }
        (module, HashSet::new())
    };

    if let Some(value) = nd_policy_override {
        module
            .meta
            .insert("nd_policy".to_string(), value.to_string());
    }
    let diagnostics = validate_module_with_imports(&module, &imported_roots);
    if !diagnostics.is_empty() {
        let rendered = format_diagnostics(&diagnostics);
        if !has_errors(&diagnostics) {
            eprintln!("Semantic validation warnings:\n{}", rendered);
        } else {
            bail!("Semantic validation failed:\n{}", rendered);
        }
    }
    Ok(from_ast(module))
}

fn is_project_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.ends_with(".sculpt.json"))
        .unwrap_or(false)
}

fn load_project_context(project_file: &Path) -> Result<ProjectContext> {
    let project_text = fs::read_to_string(project_file)
        .with_context(|| format!("Failed to read project file {}", project_file.display()))?;
    let spec: SculptProjectFile = serde_json::from_str(&project_text)
        .with_context(|| format!("Invalid project file JSON {}", project_file.display()))?;
    if spec.modules.is_empty() {
        bail!("Project file {} has no modules", project_file.display());
    }

    let base_dir = project_file
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut modules = HashMap::new();
    for rel in &spec.modules {
        let path = base_dir.join(rel);
        let src = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read module source {}", path.display()))?;
        let module = parse_source(&src)
            .with_context(|| format!("Failed to parse module source {}", path.display()))?;
        if modules
            .insert(module.name.clone(), (path.clone(), module))
            .is_some()
        {
            bail!("Duplicate module namespace in project: {}", rel);
        }
    }

    let entry_module = if let Some(entry) = spec.entry {
        entry
    } else if modules.len() == 1 {
        modules
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Project module index empty"))?
    } else {
        bail!(
            "Project file {} must define 'entry' when more than one module is present",
            project_file.display()
        );
    };
    if !modules.contains_key(&entry_module) {
        bail!("Project entry '{}' not found in modules list", entry_module);
    }

    let _project_name = spec.name.unwrap_or_else(|| {
        project_file
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".sculpt.json"))
            .unwrap_or("sculpt")
            .to_string()
    });

    Ok(ProjectContext {
        entry_module,
        modules,
    })
}

fn resolve_imported_module_roots_project(
    module: &crate::ast::Module,
    modules: &HashMap<String, (PathBuf, crate::ast::Module)>,
) -> Result<HashSet<String>> {
    let mut roots = HashSet::new();
    let mut visited = HashSet::new();
    resolve_imported_module_roots_project_inner(module, modules, &mut roots, &mut visited)?;
    Ok(roots)
}

fn resolve_imported_module_roots_project_inner(
    module: &crate::ast::Module,
    modules: &HashMap<String, (PathBuf, crate::ast::Module)>,
    roots: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) -> Result<()> {
    for import in &module.imports {
        let imported = modules
            .get(&import.path)
            .map(|(_, m)| m)
            .ok_or_else(|| anyhow::anyhow!("Unknown imported module '{}'", import.path))?;
        if !visited.insert(import.path.clone()) {
            continue;
        }

        if let Some(alias) = &import.alias {
            roots.insert(alias.clone());
        } else {
            let module_root = import
                .path
                .split('.')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if module_root.is_empty() {
                bail!(
                    "Imported module '{}' has invalid namespace root",
                    import.path
                );
            }
            roots.insert(module_root);
        }
        resolve_imported_module_roots_project_inner(imported, modules, roots, visited)?;
    }
    Ok(())
}

fn select_ai_provider(
    provider_override: Option<String>,
    model_override: Option<String>,
    strict: bool,
) -> Result<(AiProvider, ProviderInfo)> {
    let config = load_config();
    let provider_name = provider_override
        .or_else(|| config.provider)
        .ok_or_else(|| {
            anyhow::anyhow!("Provider required. Use --provider or set in sculpt.config.json")
        })?;

    match provider_name.as_str() {
        "openai" => {
            let key = env::var("OPENAI_API_KEY")
                .ok()
                .or_else(|| config.openai.as_ref().and_then(|c| c.api_key.clone()));
            if let Some(api_key) = key {
                let model_name = model_override
                    .or_else(|| config.openai.and_then(|c| c.model))
                    .unwrap_or_else(|| "gpt-4.1".to_string());
                Ok((
                    AiProvider::OpenAI {
                        api_key,
                        model: model_name.clone(),
                    },
                    ProviderInfo {
                        name: "openai".to_string(),
                        model: model_name,
                    },
                ))
            } else if strict {
                bail!("OpenAI provider selected but no API key provided");
            } else {
                eprintln!(
                    "Warning: OpenAI provider selected but no API key found. Falling back to stub."
                );
                Ok((
                    AiProvider::Stub,
                    ProviderInfo {
                        name: "stub".to_string(),
                        model: "stub".to_string(),
                    },
                ))
            }
        }
        "anthropic" => {
            let key = env::var("ANTHROPIC_API_KEY")
                .ok()
                .or_else(|| config.anthropic.as_ref().and_then(|c| c.api_key.clone()));
            if let Some(api_key) = key {
                let model_name = model_override
                    .or_else(|| config.anthropic.as_ref().and_then(|c| c.model.clone()))
                    .unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
                Ok((
                    AiProvider::Anthropic {
                        api_key,
                        model: model_name.clone(),
                    },
                    ProviderInfo {
                        name: "anthropic".to_string(),
                        model: model_name,
                    },
                ))
            } else if strict {
                bail!("Anthropic provider selected but no API key provided");
            } else {
                eprintln!("Warning: Anthropic provider selected but no API key found. Falling back to stub.");
                Ok((
                    AiProvider::Stub,
                    ProviderInfo {
                        name: "stub".to_string(),
                        model: "stub".to_string(),
                    },
                ))
            }
        }
        "gemini" => {
            let key = env::var("GEMINI_API_KEY")
                .ok()
                .or_else(|| config.gemini.as_ref().and_then(|c| c.api_key.clone()));
            if let Some(api_key) = key {
                let model_name = model_override
                    .or_else(|| config.gemini.as_ref().and_then(|c| c.model.clone()))
                    .unwrap_or_else(|| "gemini-2.5-pro".to_string());
                Ok((
                    AiProvider::Gemini {
                        api_key,
                        model: model_name.clone(),
                    },
                    ProviderInfo {
                        name: "gemini".to_string(),
                        model: model_name,
                    },
                ))
            } else if strict {
                bail!("Gemini provider selected but no API key provided");
            } else {
                eprintln!(
                    "Warning: Gemini provider selected but no API key found. Falling back to stub."
                );
                Ok((
                    AiProvider::Stub,
                    ProviderInfo {
                        name: "stub".to_string(),
                        model: "stub".to_string(),
                    },
                ))
            }
        }
        "stub" => Ok((
            AiProvider::Stub,
            ProviderInfo {
                name: "stub".to_string(),
                model: "stub".to_string(),
            },
        )),
        other => bail!("Unknown AI provider: {}", other),
    }
}

fn build_target_spec_from_value(spec: &Value) -> Result<TargetSpec> {
    let standard_ir = spec
        .get("standard_ir")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let schema = spec.get("schema").cloned().unwrap_or(Value::Null);
    let extensions = spec
        .get("extensions")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
    if standard_ir.is_empty() || schema.is_null() {
        bail!("Target describe missing standard_ir or schema");
    }
    Ok(TargetSpec {
        standard_ir,
        schema,
        extensions,
    })
}

fn generate_with_convergence(
    ai_provider: AiProvider,
    provider: Option<String>,
    model: Option<String>,
    strict: bool,
    sculpt_ir_value: &Value,
    spec: &TargetSpec,
    nondet: &str,
    previous_target_ir: Option<&Value>,
    layout_required: bool,
    controls: &ConvergenceControls,
) -> Result<(Value, Option<DebugCapture>)> {
    let mut attempt = 1u32;
    let mut provider_once = Some(ai_provider);
    let mut last_error: Option<anyhow::Error> = None;

    while attempt <= controls.max_iterations {
        let provider_for_attempt = if let Some(p) = provider_once.take() {
            p
        } else {
            select_ai_provider(provider.clone(), model.clone(), strict)?.0
        };
        match generate_target_ir(
            provider_for_attempt,
            sculpt_ir_value,
            spec,
            nondet,
            previous_target_ir,
            layout_required,
            controls,
        ) {
            Ok(result) => return Ok(result),
            Err(err) => {
                last_error = Some(err);
                attempt += 1;
            }
        }
    }

    match controls.fallback {
        FallbackMode::Fail => {
            let err_text = last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown error".to_string());
            bail!(
                "LLM compile failed after {} attempt(s) and fallback=fail: {}",
                controls.max_iterations,
                err_text
            );
        }
        FallbackMode::Stub => {
            eprintln!(
                "Warning: LLM compile failed after {} attempt(s). Applying fallback=stub.",
                controls.max_iterations
            );
            generate_target_ir(
                AiProvider::Stub,
                sculpt_ir_value,
                spec,
                nondet,
                previous_target_ir,
                layout_required,
                controls,
            )
        }
        FallbackMode::Replay => {
            if let Some(prev) = previous_target_ir {
                eprintln!(
                    "Warning: LLM compile failed after {} attempt(s). Applying fallback=replay.",
                    controls.max_iterations
                );
                Ok((prev.clone(), None))
            } else {
                let err_text = last_error
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "unknown error".to_string());
                bail!(
          "LLM compile failed after {} attempt(s) and fallback=replay had no previous target IR: {}",
          controls.max_iterations,
          err_text
        );
            }
        }
    }
}

fn deterministic_build(
    target: &str,
    target_ir: &TargetIr,
    target_ir_value: &Value,
    input: &Path,
    dist_dir: &Path,
) -> Result<()> {
    match resolve_target(target) {
        TargetKind::Cli => {
            emit_cli(target_ir, dist_dir)?;
        }
        TargetKind::Web => {
            emit_web(target_ir, dist_dir)?;
        }
        TargetKind::Gui => {
            emit_gui(target_ir, dist_dir)?;
        }
        TargetKind::External(name) => {
            run_external_target(
                &name,
                &load_ir(input, None)?,
                None,
                Some(target_ir_value),
                dist_dir,
                input,
                None,
                "build",
            )?;
        }
    }
    Ok(())
}

fn read_previous_target_ir(input: &Path) -> Option<Value> {
    let path = dist_dir(input).join("target.ir.json");
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

fn dist_dir(input: &Path) -> PathBuf {
    dist_dir_for_input(input)
}

fn load_config() -> Config {
    let path = Path::new("sculpt.config.json");
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(cfg) = serde_json::from_str::<Config>(&data) {
            return cfg;
        }
    }
    Config::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::from_ast;
    use crate::parser::parse_source;
    use std::fs::File;
    use std::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workspace() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "sculpt_cli_test_{}_{}_{}",
            std::process::id(),
            stamp,
            seq
        ));
        fs::create_dir_all(&dir).expect("create temp workspace");
        dir
    }

    fn no_retention() -> CleanRetention {
        CleanRetention {
            max_age_days: None,
            keep_latest: None,
            max_size_mb: None,
        }
    }

    #[test]
    fn clean_all_removes_dist() {
        let ws = temp_workspace();
        fs::create_dir_all(ws.join("dist/hello_world")).expect("make dist");
        fs::write(ws.join("dist/hello_world/main.js"), "ok").expect("write file");

        clean_impl(&ws, None, true, no_retention()).expect("clean all");
        assert!(!ws.join("dist").exists());

        let _ = fs::remove_dir_all(ws);
    }

    #[test]
    fn clean_input_removes_only_matching_script_dist() {
        let ws = temp_workspace();
        fs::create_dir_all(ws.join("dist/a")).expect("make dist a");
        fs::create_dir_all(ws.join("dist/b")).expect("make dist b");
        fs::write(ws.join("dist/a/main.js"), "a").expect("write a");
        fs::write(ws.join("dist/b/main.js"), "b").expect("write b");

        clean_impl(&ws, Some(Path::new("a.sculpt")), false, no_retention()).expect("clean input");
        assert!(!ws.join("dist/a").exists());
        assert!(ws.join("dist/b").exists());

        let _ = fs::remove_dir_all(ws);
    }

    #[test]
    fn clean_retention_keep_latest_keeps_newest_only() {
        let ws = temp_workspace();
        fs::create_dir_all(ws.join("dist/a")).expect("make dist a");
        fs::create_dir_all(ws.join("dist/b")).expect("make dist b");
        fs::create_dir_all(ws.join("dist/c")).expect("make dist c");
        fs::write(ws.join("dist/a/main.js"), "a").expect("write a");
        std::thread::sleep(Duration::from_millis(5));
        fs::write(ws.join("dist/b/main.js"), "b").expect("write b");
        std::thread::sleep(Duration::from_millis(5));
        fs::write(ws.join("dist/c/main.js"), "c").expect("write c");

        clean_impl(
            &ws,
            None,
            false,
            CleanRetention {
                max_age_days: None,
                keep_latest: Some(1),
                max_size_mb: None,
            },
        )
        .expect("retention clean");

        assert!(!ws.join("dist/a").exists());
        assert!(!ws.join("dist/b").exists());
        assert!(ws.join("dist/c").exists());
        let _ = fs::remove_dir_all(ws);
    }

    #[test]
    fn clean_retention_age_removes_old_entries() {
        let ws = temp_workspace();
        fs::create_dir_all(ws.join("dist/old")).expect("make old");
        fs::create_dir_all(ws.join("dist/new")).expect("make new");
        fs::write(ws.join("dist/old/main.js"), "old").expect("write old");
        fs::write(ws.join("dist/new/main.js"), "new").expect("write new");

        let old_file = ws.join("dist/old/main.js");
        let old_dir = ws.join("dist/old");
        let old_time = SystemTime::now() - Duration::from_secs(3 * 24 * 60 * 60);
        File::options()
            .write(true)
            .open(&old_file)
            .expect("open old file")
            .set_modified(old_time)
            .expect("set modified");
        let _ = File::open(&old_dir)
            .and_then(|f| f.set_modified(old_time))
            .ok();

        clean_impl(
            &ws,
            None,
            false,
            CleanRetention {
                max_age_days: Some(1),
                keep_latest: None,
                max_size_mb: None,
            },
        )
        .expect("retention clean");

        assert!(!ws.join("dist/old").exists());
        assert!(ws.join("dist/new").exists());
        let _ = fs::remove_dir_all(ws);
    }

    #[test]
    fn clean_rejects_all_with_retention_options() {
        let ws = temp_workspace();
        let err = clean_impl(
            &ws,
            None,
            true,
            CleanRetention {
                max_age_days: Some(1),
                keep_latest: None,
                max_size_mb: None,
            },
        )
        .expect_err("must fail");
        assert!(format!("{err}").contains("--all"));
        let _ = fs::remove_dir_all(ws);
    }

    #[test]
    fn auto_clean_policy_from_config_requires_enabled_and_limits() {
        let mut cfg = Config::default();
        assert!(auto_clean_retention_from_config(&cfg).is_none());
        cfg.clean = Some(CleanConfig {
            auto: Some(true),
            max_age_days: Some(7),
            keep_latest: None,
            max_size_mb: None,
        });
        let retention = auto_clean_retention_from_config(&cfg).expect("retention");
        assert_eq!(retention.max_age_days, Some(7));
    }

    #[test]
    fn validate_retention_rejects_zero_values() {
        let err = validate_retention(CleanRetention {
            max_age_days: None,
            keep_latest: Some(0),
            max_size_mb: None,
        })
        .expect_err("must fail");
        assert!(format!("{err}").contains("keep-latest"));
    }

    #[test]
    fn gate_eval_handles_lower_is_better_with_delta() {
        let criterion = GateCriterion {
            id: "G1".to_string(),
            description: "regression count".to_string(),
            sculpt: 1.0,
            vibe: 4.0,
            operator: "sculpt_lt_vibe".to_string(),
            min_delta: Some(2.0),
        };
        let eval = evaluate_gate_criterion(&criterion).expect("eval");
        assert!(eval.passed);
    }

    #[test]
    fn gate_eval_fails_equal_when_values_differ() {
        let criterion = GateCriterion {
            id: "G2".to_string(),
            description: "quality parity".to_string(),
            sculpt: 1.0,
            vibe: 0.0,
            operator: "equal".to_string(),
            min_delta: None,
        };
        let eval = evaluate_gate_criterion(&criterion).expect("eval");
        assert!(!eval.passed);
    }

    #[test]
    fn required_output_contract_passes_when_writer_calls_match_meta() {
        let src = r#"@meta target=cli
@meta required_outputs="reconciliation_report.json,exceptions.csv"
module(App.Core):
  flow(Main):
    start > Exit
    state(Exit):
      terminate
    end
  end
end
"#;
        let module = parse_source(src).expect("parse");
        let ir = from_ast(module);
        let target_ir: TargetIr = serde_json::from_value(serde_json::json!({
            "type":"cli-ir",
            "version":1,
            "state": {
                "reportPath":"reconciliation_report.json",
                "exceptionsPath":"exceptions.csv"
            },
            "views":{},
            "flow":{"start":"Exit","transitions":{"Exit":{}}},
            "extensions":{
                "runtimeRules":[
                    {"name":"a","assign":[{"value":{"call":{"name":"writeJson","args":[{"value":{"ident":"reportPath"}},{"value":{"k":"v"}}]}}}],"emit":[]},
                    {"name":"b","assign":[{"value":{"call":{"name":"writeCsv","args":[{"value":{"ident":"exceptionsPath"}},{"value":[]}]}}}],"emit":[]}
                ]
            }
        }))
        .expect("target ir");
        validate_required_output_contract(&ir, &target_ir, "cli").expect("contract valid");
    }

    #[test]
    fn required_output_contract_fails_when_writer_call_missing() {
        let src = r#"@meta target=cli
@meta required_outputs="exceptions.csv"
module(App.Core):
  flow(Main):
    start > Exit
    state(Exit):
      terminate
    end
  end
end
"#;
        let module = parse_source(src).expect("parse");
        let ir = from_ast(module);
        let target_ir: TargetIr = serde_json::from_value(serde_json::json!({
            "type":"cli-ir",
            "version":1,
            "state": {},
            "views":{},
            "flow":{"start":"Exit","transitions":{"Exit":{}}},
            "extensions":{"runtimeRules":[]}
        }))
        .expect("target ir");
        let err = validate_required_output_contract(&ir, &target_ir, "cli").expect_err("must fail");
        assert!(format!("{err}").contains("C911"));
    }

    #[test]
    fn benchmark_provider_chain_openai_falls_back_to_gemini_then_stub() {
        let chain = benchmark_provider_chain(Some("openai".to_string()), false);
        assert_eq!(chain, vec!["openai", "gemini", "stub"]);
    }

    #[test]
    fn benchmark_provider_chain_strict_keeps_requested_only() {
        let chain = benchmark_provider_chain(Some("openai".to_string()), true);
        assert_eq!(chain, vec!["openai"]);
    }

    #[test]
    fn provider_unavailable_detection_handles_quota_message() {
        assert!(is_provider_unavailable_error(
            "OpenAI error: status 429 Too Many Requests code=insufficient_quota"
        ));
        assert!(!is_provider_unavailable_error(
            "C901: Unknown state reference in deterministic path"
        ));
    }

    #[test]
    fn verify_gui_artifacts_accepts_swift_binary_layout() {
        let ws = temp_workspace();
        let dist = ws.join("dist/gui_case");
        fs::create_dir_all(dist.join("gui/.build/release")).expect("create gui build dir");
        fs::write(dist.join("target.ir.json"), "{}").expect("write target");
        fs::write(dist.join("ir.json"), "{}").expect("write ir");
        fs::write(dist.join("nondet.report"), "ok").expect("write report");
        fs::write(dist.join("gui/.build/release/SculptGui"), "").expect("write binary marker");

        verify_build_artifacts("gui", &dist).expect("gui artifacts valid");
        let _ = fs::remove_dir_all(ws);
    }

    #[test]
    fn verify_gui_artifacts_accepts_python_entry_layout() {
        let ws = temp_workspace();
        let dist = ws.join("dist/gui_case_py");
        fs::create_dir_all(dist.join("gui")).expect("create gui dir");
        fs::write(dist.join("target.ir.json"), "{}").expect("write target");
        fs::write(dist.join("ir.json"), "{}").expect("write ir");
        fs::write(dist.join("nondet.report"), "ok").expect("write report");
        fs::write(dist.join("gui/main.py"), "print('ok')").expect("write python entry");

        verify_build_artifacts("gui", &dist).expect("gui artifacts valid");
        let _ = fs::remove_dir_all(ws);
    }
}
