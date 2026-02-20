use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use crate::ai::{generate_target_ir, AiProvider, DebugCapture, TargetSpec};
use crate::build_meta::{dist_dir_for_input, now_unix_ms, write_build_meta, BuildMeta, TokenUsage};
use crate::contracts::{parse_target_contract, validate_module_against_contract};
use crate::convergence::{ConvergenceControls, FallbackMode};
use crate::freeze::{create_lock, read_lock, verify_lock, write_lock};
use crate::ir::{from_ast, to_pretty_json, IrModule};
use crate::parser::parse_source;
use crate::report::generate_report;
use crate::semantics::{format_diagnostics, validate_module};
use crate::target_ir::{from_json_value, TargetIr};
use crate::targets::{
    describe_target, emit_cli, emit_gui, emit_web, list_targets, resolve_target, run_cli,
    run_external_target, run_gui, run_web, TargetKind,
};
use serde_json::Value;

#[derive(Parser)]
#[command(
    name = "sculpt",
    version,
    about = "SCULPT compiler — (C) 2026 byte5 GmbH",
    after_help = "TUI: run `sculpt` with no arguments"
)]
pub struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Examples,
    Gate {
        #[command(subcommand)]
        cmd: GateCommand,
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
        #[arg(long = "nd-policy", value_parser = ["strict", "magic"])]
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
        #[arg(long = "nd-policy", value_parser = ["strict", "magic"])]
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
    },
}

#[derive(Subcommand)]
pub enum TargetCommand {
    List,
    Describe {
        #[arg(long)]
        target: String,
    },
}

#[derive(Subcommand)]
pub enum GateCommand {
    Check { gate_file: PathBuf },
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

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Examples => write_examples(),
        Command::Gate { cmd } => match cmd {
            GateCommand::Check { gate_file } => gate_check(&gate_file),
        },
        Command::Auth { cmd } => match cmd {
            AuthCommand::Check { provider, verify } => auth_check(&provider, verify),
        },
        Command::Target { cmd } => match cmd {
            TargetCommand::List => target_list(),
            TargetCommand::Describe { target } => target_describe(&target),
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
        Command::Clean { input, all } => clean_cmd(input.as_deref(), all),
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

fn write_examples() -> Result<()> {
    let examples_dir = Path::new("examples");
    fs::create_dir_all(examples_dir)?;

    let files: &[(&str, &str)] = &[
        (
            "getting-started/hello_world.sculpt",
            r#"# Hello World (tradition kept)
# Minimal deterministic example with no ND.

module(HelloWorld):

  flow(App):
    start > Show

    state(Show):
      render text("Hallo", color: "yellow")
      render text("Welt", color: "blue")
    end
  end

end
"#,
        ),
        (
            "getting-started/native_window.sculpt",
            r#"# Native Window Demo (macOS GUI)
# Goal: show a real window with text + button.

@meta target=gui layout=explicit

module(NativeWindow):

  flow(App):
    start > Main

    state(Main):
      render text("SCULPT Native Demo", color: "yellow")
      render text("Click the button to open an OK modal", color: "blue")
      render button("Open OK", action: "modal.ok")
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
      run Loop
      on done > Title
    end

    state(Loop):
      on tick > Loop
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
      run Loop
      on done > Title
    end

    state(Loop):
      on tick > Loop
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
      when hitWall(snake, width, height)
        emit done
      end
      when hitSelf(snake)
        emit done
      end
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
            "games/breakout_cli.sculpt",
            r#"# Breakout (CLI)
# Demonstrates a playable arcade loop with clear game-state rules and constrained ND for level layout.

@meta target=cli
@meta nd_budget=24
@meta confidence=0.9

module(BreakoutCLI):

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

  rule(launchBall):
    on key(Space):
      launched = 1
    end
  end

  rule(moveLeft):
    on key(A):
      paddleX = movePaddle(paddleX, "left", width, paddleWidth)
    end
  end

  rule(moveRight):
    on key(D):
      paddleX = movePaddle(paddleX, "right", width, paddleWidth)
    end
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
    when 0 >= lives:
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
      render text("Session closed. Stay calm and log your actions.", color: "green")
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
      on key(A) > Approved
      on key(R) > Rejected
      on key(N) > NeedInfo
      on key(Esc) > Inbox
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
    approvalLimit = 1000
    queueSize = 12
    selectedRequest = "EX-2048"
    riskScore = 18
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
@meta layout=explicit
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
    Ok(())
}

fn clean_cmd(input: Option<&Path>, all: bool) -> Result<()> {
    let root = std::env::current_dir()?;
    clean_impl(&root, input, all)
}

fn clean_impl(root: &Path, input: Option<&Path>, all: bool) -> Result<()> {
    if all {
        let dist = root.join("dist");
        if dist.exists() {
            fs::remove_dir_all(&dist)?;
            println!("Removed {}", dist.display());
        } else {
            println!("Nothing to clean.");
        }
        return Ok(());
    }

    let Some(input) = input else {
        bail!("Provide an input file or use --all");
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
    format!(
        "  {} {} {}",
        style_dim(&format!("{idx}.")),
        label,
        style_accent(status)
    )
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
    println!("Available targets:");
    for t in targets {
        println!("  {}", t);
    }
    Ok(())
}

fn gate_check(gate_file: &Path) -> Result<()> {
    let raw = fs::read_to_string(gate_file)
        .with_context(|| format!("Failed to read gate file {}", gate_file.display()))?;
    let spec: GateSpec = serde_json::from_str(&raw)
        .with_context(|| format!("Invalid gate JSON in {}", gate_file.display()))?;

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
    let src = fs::read_to_string(input).with_context(|| format!("Failed to read {:?}", input))?;
    let mut module = parse_source(&src)?;
    if let Some(value) = nd_policy_override {
        module
            .meta
            .insert("nd_policy".to_string(), value.to_string());
    }
    let diagnostics = validate_module(&module);
    if !diagnostics.is_empty() {
        bail!(
            "Semantic validation failed:\n{}",
            format_diagnostics(&diagnostics)
        );
    }
    Ok(from_ast(module))
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workspace() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("sculpt_cli_test_{}", stamp));
        fs::create_dir_all(&dir).expect("create temp workspace");
        dir
    }

    #[test]
    fn clean_all_removes_dist() {
        let ws = temp_workspace();
        fs::create_dir_all(ws.join("dist/hello_world")).expect("make dist");
        fs::write(ws.join("dist/hello_world/main.js"), "ok").expect("write file");

        clean_impl(&ws, None, true).expect("clean all");
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

        clean_impl(&ws, Some(Path::new("a.sculpt")), false).expect("clean input");
        assert!(!ws.join("dist/a").exists());
        assert!(ws.join("dist/b").exists());

        let _ = fs::remove_dir_all(ws);
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
}
