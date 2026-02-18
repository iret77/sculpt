use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use crate::ai::{generate_target_ir, AiProvider, TargetSpec};
use crate::freeze::{create_lock, read_lock, verify_lock, write_lock};
use crate::ir::{from_ast, to_pretty_json};
use crate::parser::parse_source;
use crate::report::generate_report;
use crate::target_ir::{from_json_value, TargetIr};
use crate::targets::{
  emit_cli,
  emit_gui,
  emit_web,
  list_targets,
  describe_target,
  resolve_target,
  run_cli,
  run_gui,
  run_external_target,
  run_web,
  TargetKind,
};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "sculpt", version, about = "SCULPT MVP compiler")]
pub struct Cli {
  #[command(subcommand)]
  cmd: Command,
}

#[derive(Subcommand)]
pub enum Command {
  Examples,
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
    target: String,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    strict_provider: bool,
  },
  Freeze {
    input: PathBuf,
    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    strict_provider: bool,
    #[arg(long)]
    target: String,
  },
  Replay {
    input: PathBuf,
    #[arg(long)]
    target: String,
  },
  Run {
    input: PathBuf,
    #[arg(long)]
    target: String,
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
    Command::Auth { cmd } => match cmd {
      AuthCommand::Check { provider, verify } => auth_check(&provider, verify),
    },
    Command::Target { cmd } => match cmd {
      TargetCommand::List => target_list(),
      TargetCommand::Describe { target } => target_describe(&target),
    },
    Command::Build { input, target, provider, model, strict_provider } => {
      build(&input, &target, provider, model, strict_provider)
    }
    Command::Freeze { input, provider, model, strict_provider, target } => {
      freeze(&input, provider, model, strict_provider, &target)
    }
    Command::Replay { input, target } => replay(&input, &target),
    Command::Run { input, target } => run_cmd(&input, &target),
  }
}

fn write_examples() -> Result<()> {
  let examples_dir = Path::new("examples");
  fs::create_dir_all(examples_dir)?;
  let hello_world = r#"# Hello World (tradition kept)
# Minimal deterministic example with no ND.

module(HelloWorld)

  flow(App)
    start > Show

    state(Show)
      render text("Hallo", color: "yellow")
      render text("Welt", color: "blue")
      terminate
    end
  end

end
"#;

  let snake_high_nd = r#"# Snake (High ND)
# Goal: minimal code, large solution space.
# Most of the game design is delegated to the LLM.

module(SnakeHighND)

  # Main flow
  flow(Game)
    start > Title

    state(Title)
      render text("SNAKE", color: "yellow")
      render text("Press Enter", color: "blue")
      on key(Enter) > Play
      on key(Esc)   > Exit
    end

    state(Play)
      run Loop
      on done > Title
    end

    state(Loop)
      on tick > Loop
      on key(Esc) > Exit
    end

    state(Exit)
      terminate
    end
  end

  # Minimal state (intentionally sparse)
  state()
    speedMs = 160
    score = 0
  end

  rule(tick)
    on tick
      score += 1
    end
  end

  rule(finish)
    when score >= 10
      emit done
    end
  end

  # High-ND block: most of the game definition is delegated to the LLM
  nd(designSnake)
    propose game("snake", size: 10)
    satisfy(
      playable(),
      funToLearn(),
      usesKeys("WASD"),
      loopedGameplay()
    )
  end
end
"#;

  let snake_low_nd = r#"# Snake (Low ND)
# Goal: highly specified rules so the solution space is narrow.
# ND is reduced to a tiny UI-theme choice.

module(SnakeLowND)

  # Main flow
  flow(Game)
    start > Title

    state(Title)
      render text("SNAKE", color: "yellow")
      render text("Enter = Start, Esc = Quit", color: "blue")
      on key(Enter) > Play
      on key(Esc)   > Exit
    end

    state(Play)
      run Loop
      on done > Title
    end

    state(Loop)
      on tick > Loop
      on key(Esc) > Exit
    end

    state(Exit)
      terminate
    end
  end

  # Deterministic configuration and initial game state
  state()
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
  rule(tick)
    on tick
      score += 1
      stepIndex += 1
    end
  end

  # Deterministic input handling
  rule(inputUp)
    on key(W)
      pendingDirection = "up"
    end
  end

  rule(inputDown)
    on key(S)
      pendingDirection = "down"
    end
  end

  rule(inputLeft)
    on key(A)
      pendingDirection = "left"
    end
  end

  rule(inputRight)
    on key(D)
      pendingDirection = "right"
    end
  end

  # Deterministic movement and food handling
  rule(applyDirection)
    on tick
      direction = pendingDirection
      snake = moveSnake(snake, direction)
      food = nextFood(foodSequence, stepIndex)
    end
  end

  rule(checkCollision)
    on tick
      when hitWall(snake, width, height)
        emit done
      end
      when hitSelf(snake)
        emit done
      end
    end
  end

  # Deterministic finish condition
  rule(finish)
    when score >= 25
      emit done
    end
  end

  # Tiny ND for cosmetics only
  nd(theme)
    propose theme("classic")
    satisfy(
      exact("classic"),
      highContrast()
    )
  end
end
"#;

  let invoice_review = r#"# Business/Web Example: Invoice Review
# Goal: clear business UI with minimal ND.

module(InvoiceReview)

  flow(App)
    start > List

    state(List)
      render text("Invoices", color: "yellow")
      render text("Enter = Open First, Esc = Quit", color: "blue")
      on key(Enter) > Detail
      on key(Esc)   > Exit
    end

    state(Detail)
      render text("Invoice #2024-001", color: "yellow")
      render text("Amount: 1,250 EUR", color: "blue")
      render text("Status: Pending", color: "blue")
      on key(A) > Approve
      on key(R) > Reject
      on key(Esc) > List
    end

    state(Approve)
      render text("Approved", color: "green")
      on key(Enter) > List
    end

    state(Reject)
      render text("Rejected", color: "red")
      on key(Enter) > List
    end

    state(Exit)
      terminate
    end
  end

  state()
    speedMs = 400
    selectedInvoice = "2024-001"
    totalInvoices = 24
  end

  # Tiny ND: layout theme only, business logic is explicit
  nd(theme)
    propose dashboardTheme("clean")
    satisfy(
      highContrast(),
      professionalTone()
    )
  end
end
"#;

  fs::write(examples_dir.join("hello_world.sculpt"), hello_world)?;
  fs::write(examples_dir.join("snake_high_nd.sculpt"), snake_high_nd)?;
  fs::write(examples_dir.join("snake_low_nd.sculpt"), snake_low_nd)?;
  fs::write(examples_dir.join("invoice_review.sculpt"), invoice_review)?;
  println!("Wrote examples/hello_world.sculpt");
  println!("Wrote examples/snake_high_nd.sculpt");
  println!("Wrote examples/snake_low_nd.sculpt");
  println!("Wrote examples/invoice_review.sculpt");
  Ok(())
}

fn build(input: &Path, target: &str, provider: Option<String>, model: Option<String>, strict: bool) -> Result<()> {
  let src = fs::read_to_string(input).with_context(|| format!("Failed to read {:?}", input))?;
  let module = parse_source(&src)?;
  let ir = from_ast(module);
  let ir_json = to_pretty_json(&ir)?;
  let nondet = generate_report(&ir);

  fs::create_dir_all("dist")?;
  fs::write("ir.json", ir_json)?;
  fs::write("nondet.report", &nondet)?;

  let spec = build_target_spec(target)?;
  let ai_provider = select_ai_provider(provider, model, strict)?;
  let sculpt_ir_value = serde_json::to_value(&ir)?;
  let previous_target_ir = read_previous_target_ir();

  let target_ir_value = generate_target_ir(ai_provider, &sculpt_ir_value, &spec, &nondet, previous_target_ir.as_ref())?;
  let target_ir = from_json_value(target_ir_value.clone())
    .map_err(|e| anyhow::anyhow!("Target IR parse error: {}", e))?;
  if target_ir.ir_type != spec.standard_ir {
    bail!("Target IR type mismatch: expected {}, got {}", spec.standard_ir, target_ir.ir_type);
  }

  fs::write("dist/target.ir.json", serde_json::to_string_pretty(&target_ir_value)?)?;
  deterministic_build(target, &target_ir, &target_ir_value, input)?;

  println!("Build complete:");
  println!("  dist/target.ir.json");
  println!("  ir.json");
  println!("  nondet.report");
  println!("  target: {}", target);
  Ok(())
}

fn freeze(input: &Path, provider: Option<String>, model: Option<String>, strict: bool, target: &str) -> Result<()> {
  let src = fs::read_to_string(input).with_context(|| format!("Failed to read {:?}", input))?;
  let module = parse_source(&src)?;
  let ir = from_ast(module);
  let nondet = generate_report(&ir);

  let spec = build_target_spec(target)?;
  let ai_provider = select_ai_provider(provider.clone(), model.clone(), strict)?;
  let sculpt_ir_value = serde_json::to_value(&ir)?;
  let previous_target_ir = read_previous_target_ir();

  let target_ir_value = generate_target_ir(ai_provider, &sculpt_ir_value, &spec, &nondet, previous_target_ir.as_ref())?;
  let target_ir = from_json_value(target_ir_value.clone())
    .map_err(|e| anyhow::anyhow!("Target IR parse error: {}", e))?;
  if target_ir.ir_type != spec.standard_ir {
    bail!("Target IR type mismatch: expected {}, got {}", spec.standard_ir, target_ir.ir_type);
  }

  let provider_name = provider.unwrap_or_else(|| "openai".to_string());
  let model_name = model.unwrap_or_else(|| "gpt-4.1".to_string());
  let lock = create_lock(&ir, &provider_name, target, &target_ir_value, &model_name)?;
  write_lock(Path::new("sculpt.lock"), &lock)?;

  fs::create_dir_all("dist")?;
  fs::write("dist/target.ir.json", serde_json::to_string_pretty(&target_ir_value)?)?;
  fs::write("ir.json", to_pretty_json(&ir)?)?;
  fs::write("nondet.report", &nondet)?;

  deterministic_build(target, &target_ir, &target_ir_value, input)?;

  println!("Freeze complete:");
  println!("  sculpt.lock");
  println!("  dist/target.ir.json");
  println!("  ir.json");
  println!("  nondet.report");
  println!("  target: {}", target);
  Ok(())
}

fn replay(input: &Path, target: &str) -> Result<()> {
  let src = fs::read_to_string(input).with_context(|| format!("Failed to read {:?}", input))?;
  let module = parse_source(&src)?;
  let ir = from_ast(module);
  let lock = read_lock(Path::new("sculpt.lock"))?;
  verify_lock(&ir, &lock)?;

  let target_ir_value = lock.target_ir.clone();
  let target_ir = from_json_value(target_ir_value.clone())
    .map_err(|e| anyhow::anyhow!("Target IR parse error: {}", e))?;

  fs::create_dir_all("dist")?;
  fs::write("dist/target.ir.json", serde_json::to_string_pretty(&target_ir_value)?)?;
  deterministic_build(target, &target_ir, &target_ir_value, input)?;
  fs::write("ir.json", to_pretty_json(&ir)?)?;
  fs::write("nondet.report", generate_report(&ir))?;

  println!("Replay complete:");
  println!("  dist/target.ir.json");
  println!("  ir.json");
  println!("  nondet.report");
  println!("  target: {}", target);
  Ok(())
}

fn run_cmd(input: &Path, target: &str) -> Result<()> {
  match resolve_target(target) {
    TargetKind::Cli => run_cli(Path::new("dist")),
    TargetKind::Web => run_web(Path::new("dist")),
    TargetKind::Gui => run_gui(Path::new("dist")),
    TargetKind::External(name) => {
      run_external_target(&name, &load_ir(input)?, None, None, Path::new("dist"), input, None, "run")?;
      Ok(())
    }
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
      let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);
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

fn load_ir(input: &Path) -> Result<crate::ir::IrModule> {
  let src = fs::read_to_string(input).with_context(|| format!("Failed to read {:?}", input))?;
  let module = parse_source(&src)?;
  Ok(from_ast(module))
}

fn select_ai_provider(
  provider_override: Option<String>,
  model_override: Option<String>,
  strict: bool,
) -> Result<AiProvider> {
  let config = load_config();
  let provider_name = provider_override
    .or_else(|| config.provider)
    .ok_or_else(|| anyhow::anyhow!("Provider required. Use --provider or set in sculpt.config.json"))?;

  match provider_name.as_str() {
    "openai" => {
      let key = env::var("OPENAI_API_KEY")
        .ok()
        .or_else(|| config.openai.as_ref().and_then(|c| c.api_key.clone()));
      if let Some(api_key) = key {
        let model_name = model_override
          .or_else(|| config.openai.and_then(|c| c.model))
          .unwrap_or_else(|| "gpt-4.1".to_string());
        Ok(AiProvider::OpenAI { api_key, model: model_name })
      } else if strict {
        bail!("OpenAI provider selected but no API key provided");
      } else {
        eprintln!("Warning: OpenAI provider selected but no API key found. Falling back to stub.");
        Ok(AiProvider::Stub)
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
        Ok(AiProvider::Anthropic { api_key, model: model_name })
      } else if strict {
        bail!("Anthropic provider selected but no API key provided");
      } else {
        eprintln!("Warning: Anthropic provider selected but no API key found. Falling back to stub.");
        Ok(AiProvider::Stub)
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
        Ok(AiProvider::Gemini { api_key, model: model_name })
      } else if strict {
        bail!("Gemini provider selected but no API key provided");
      } else {
        eprintln!("Warning: Gemini provider selected but no API key found. Falling back to stub.");
        Ok(AiProvider::Stub)
      }
    }
    "stub" => Ok(AiProvider::Stub),
    other => bail!("Unknown AI provider: {}", other),
  }
}

fn build_target_spec(target: &str) -> Result<TargetSpec> {
  let spec = describe_target(target)?;
  let standard_ir = spec.get("standard_ir").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let schema = spec.get("schema").cloned().unwrap_or(Value::Null);
  let extensions = spec.get("extensions").cloned().unwrap_or_else(|| Value::Object(serde_json::Map::new()));
  if standard_ir.is_empty() || schema.is_null() {
    bail!("Target describe missing standard_ir or schema");
  }
  Ok(TargetSpec { standard_ir, schema, extensions })
}

fn deterministic_build(target: &str, target_ir: &TargetIr, target_ir_value: &Value, input: &Path) -> Result<()> {
  match resolve_target(target) {
    TargetKind::Cli => {
      emit_cli(target_ir, Path::new("dist"))?;
    }
    TargetKind::Web => {
      emit_web(target_ir, Path::new("dist"))?;
    }
    TargetKind::Gui => {
      emit_gui(target_ir, Path::new("dist"))?;
    }
    TargetKind::External(name) => {
      run_external_target(&name, &load_ir(input)?, None, Some(target_ir_value), Path::new("dist"), input, None, "build")?;
    }
  }
  Ok(())
}

fn read_previous_target_ir() -> Option<Value> {
  let path = Path::new("dist/target.ir.json");
  let data = fs::read_to_string(path).ok()?;
  serde_json::from_str(&data).ok()
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
