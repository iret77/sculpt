use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use serde_json::{json, Value};

use crate::build_meta::TokenUsage;
use crate::convergence::ConvergenceControls;
use crate::llm_ir::{compact_schema_for, normalize_llm_ir};

pub enum AiProvider {
  OpenAI { api_key: String, model: String },
  Anthropic { api_key: String, model: String },
  Gemini { api_key: String, model: String },
  Stub,
}

pub struct TargetSpec {
  pub standard_ir: String,
  pub schema: Value,
  pub extensions: Value,
}

pub struct DebugCapture {
  pub prompt: String,
  pub raw_output: String,
  pub llm_ms: u128,
  pub token_usage: Option<TokenUsage>,
}

pub fn generate_target_ir(
  provider: AiProvider,
  sculpt_ir: &Value,
  target_spec: &TargetSpec,
  nondet_report: &str,
  previous_target_ir: Option<&Value>,
  layout_required: bool,
  controls: &ConvergenceControls,
) -> Result<(Value, Option<DebugCapture>)> {
  match provider {
    AiProvider::OpenAI { api_key, model } => {
      let (value, debug) =
        openai_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir, layout_required, controls)?;
      Ok((value, Some(debug)))
    }
    AiProvider::Anthropic { api_key, model } => {
      let (value, debug) =
        anthropic_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir, layout_required, controls)?;
      Ok((value, Some(debug)))
    }
    AiProvider::Gemini { api_key, model } => {
      let (value, debug) =
        gemini_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir, layout_required, controls)?;
      Ok((value, Some(debug)))
    }
    AiProvider::Stub => {
      let mut value = stub_generate(target_spec);
      patch_target_ir_with_deterministic_parts(&mut value, &target_spec.standard_ir, sculpt_ir);
      Ok((value, None))
    }
  }
}

fn stub_generate(target_spec: &TargetSpec) -> Value {
  json!({
    "type": target_spec.standard_ir,
    "version": 1,
    "state": {},
    "views": {
      "Title": [
        { "kind": "text", "text": "SCULPT", "color": "yellow" },
        { "kind": "text", "text": "stub target-ir", "color": "blue" }
      ]
    },
    "flow": {
      "start": "Title",
      "transitions": {}
    },
    "extensions": target_spec.extensions
  })
}

fn openai_generate(
  api_key: &str,
  model: &str,
  sculpt_ir: &Value,
  target_spec: &TargetSpec,
  nondet_report: &str,
  previous_target_ir: Option<&Value>,
  layout_required: bool,
  controls: &ConvergenceControls,
) -> Result<(Value, DebugCapture)> {
  let compact_schema = compact_schema_for(&target_spec.standard_ir)
    .ok_or_else(|| anyhow::anyhow!("No compact LLM schema for {}", target_spec.standard_ir))?;
  let input = build_prompt(
    sculpt_ir,
    target_spec,
    &compact_schema,
    nondet_report,
    previous_target_ir,
    layout_required,
    controls,
  )?;

  let body = json!({
    "model": model,
    "input": input,
    "text": {
      "format": {
        "type": "json_object"
      }
    }
  });

  let client = http_client()?;
  let started = Instant::now();
  let resp = client
    .post("https://api.openai.com/v1/responses")
    .bearer_auth(api_key)
    .json(&body)
    .send()?;

  if !resp.status().is_success() {
    let status = resp.status();
    let text = resp.text().unwrap_or_else(|_| "<no body>".to_string());
    bail!("OpenAI error: status {} body {}", status, text);
  }

  let value: Value = resp.json()?;
  let text = extract_output_text(&value).unwrap_or_default();
  if text.is_empty() {
    bail!("OpenAI returned empty output");
  }

  let parsed = parse_json_response(&text)?;
  let mut normalized = normalize_llm_ir(&target_spec.standard_ir, &parsed);
  patch_target_ir_with_deterministic_parts(&mut normalized, &target_spec.standard_ir, sculpt_ir);
  Ok((
    normalized,
    DebugCapture {
      prompt: input,
      raw_output: text,
      llm_ms: started.elapsed().as_millis(),
      token_usage: extract_openai_token_usage(&value),
    },
  ))
}

fn extract_output_text(value: &Value) -> Option<String> {
  let output = value.get("output")?.as_array()?;
  let mut text = String::new();
  for item in output {
    if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
      for part in content {
        if part.get("type").and_then(|t| t.as_str()) == Some("output_text") {
          if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
            text.push_str(t);
          }
        }
      }
    }
  }
  if text.is_empty() { None } else { Some(text) }
}

fn anthropic_generate(
  api_key: &str,
  model: &str,
  sculpt_ir: &Value,
  target_spec: &TargetSpec,
  nondet_report: &str,
  previous_target_ir: Option<&Value>,
  layout_required: bool,
  controls: &ConvergenceControls,
) -> Result<(Value, DebugCapture)> {
  let compact_schema = compact_schema_for(&target_spec.standard_ir)
    .ok_or_else(|| anyhow::anyhow!("No compact LLM schema for {}", target_spec.standard_ir))?;
  let input = build_prompt(
    sculpt_ir,
    target_spec,
    &compact_schema,
    nondet_report,
    previous_target_ir,
    layout_required,
    controls,
  )?;
  let body = json!({
    "model": model,
    "max_tokens": 2048,
    "system": "You are the Sculpt compiler AI. Generate target IR JSON that conforms to the provided schema. Output only JSON.",
    "messages": [
      { "role": "user", "content": input }
    ]
  });

  let client = http_client()?;
  let started = Instant::now();
  let resp = client
    .post("https://api.anthropic.com/v1/messages")
    .header("x-api-key", api_key)
    .header("anthropic-version", "2023-06-01")
    .header("content-type", "application/json")
    .json(&body)
    .send()?;

  if !resp.status().is_success() {
    let status = resp.status();
    let text = resp.text().unwrap_or_else(|_| "<no body>".to_string());
    bail!("Anthropic error: status {} body {}", status, text);
  }

  let value: Value = resp.json()?;
  let text = extract_anthropic_text(&value).unwrap_or_default();
  if text.is_empty() {
    bail!("Anthropic returned empty output");
  }
  let parsed = parse_json_response(&text)?;
  let mut normalized = normalize_llm_ir(&target_spec.standard_ir, &parsed);
  patch_target_ir_with_deterministic_parts(&mut normalized, &target_spec.standard_ir, sculpt_ir);
  Ok((
    normalized,
    DebugCapture {
      prompt: input,
      raw_output: text,
      llm_ms: started.elapsed().as_millis(),
      token_usage: extract_anthropic_token_usage(&value),
    },
  ))
}

fn gemini_generate(
  api_key: &str,
  model: &str,
  sculpt_ir: &Value,
  target_spec: &TargetSpec,
  nondet_report: &str,
  previous_target_ir: Option<&Value>,
  layout_required: bool,
  controls: &ConvergenceControls,
) -> Result<(Value, DebugCapture)> {
  let compact_schema = compact_schema_for(&target_spec.standard_ir)
    .ok_or_else(|| anyhow::anyhow!("No compact LLM schema for {}", target_spec.standard_ir))?;
  let input = build_prompt(
    sculpt_ir,
    target_spec,
    &compact_schema,
    nondet_report,
    previous_target_ir,
    layout_required,
    controls,
  )?;
  let body = json!({
    "contents": [
      { "role": "user", "parts": [ { "text": input } ] }
    ],
    "generationConfig": {
      "response_mime_type": "application/json"
    }
  });

  let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);
  let client = http_client()?;
  let started = Instant::now();
  let resp = client
    .post(url)
    .header("x-goog-api-key", api_key)
    .header("content-type", "application/json")
    .json(&body)
    .send()?;

  if !resp.status().is_success() {
    let status = resp.status();
    let text = resp.text().unwrap_or_else(|_| "<no body>".to_string());
    bail!("Gemini error: status {} body {}", status, text);
  }

  let value: Value = resp.json()?;
  let text = extract_gemini_text(&value).unwrap_or_default();
  if text.is_empty() {
    bail!("Gemini returned empty output");
  }
  let parsed = parse_json_response(&text)?;
  let mut normalized = normalize_llm_ir(&target_spec.standard_ir, &parsed);
  patch_target_ir_with_deterministic_parts(&mut normalized, &target_spec.standard_ir, sculpt_ir);
  Ok((
    normalized,
    DebugCapture {
      prompt: input,
      raw_output: text,
      llm_ms: started.elapsed().as_millis(),
      token_usage: extract_gemini_token_usage(&value),
    },
  ))
}

fn build_prompt(
  sculpt_ir: &Value,
  target_spec: &TargetSpec,
  compact_schema: &Value,
  nondet_report: &str,
  previous_target_ir: Option<&Value>,
  layout_required: bool,
  controls: &ConvergenceControls,
) -> Result<String> {
  let mut input = String::new();
  input.push_str("You are the Sculpt compiler AI. Generate target IR JSON that conforms to the provided schema.\n");
  input.push_str("Do not include explanations. Output only JSON.\n");
  input.push_str("Output must follow the compact schema exactly (positional arrays, no extra keys).\n");
  input.push_str("Format:\n");
  input.push_str("  u = [ [viewName, [ [kind,text,color,x,y,action,style], ... ] ], ... ]\n");
  input.push_str("  f = [ start, [ [from, [ [event,target], ... ] ], ... ] ]\n");
  input.push_str("  w = [title,width,height]\n\n");
  input.push_str("  l = [ [viewName, [padding,spacing,align,background] ], ... ]\n");
  input.push_str("Action semantics:\n");
  input.push_str("  action=\"modal.ok\" means show a simple OK modal when the button is clicked.\n\n");
  if layout_required {
    input.push_str("Layout is REQUIRED: include l with explicit layout for each view.\n\n");
  }
  input.push_str("CONVERGENCE_CONTROLS:\n");
  input.push_str(&format!(
    "nd_budget={}\nconfidence={}\nmax_iterations={}\nfallback={:?}\n\n",
    controls
      .nd_budget
      .map(|v| v.to_string())
      .unwrap_or_else(|| "unset".to_string()),
    controls
      .confidence
      .map(|v| format!("{v:.2}"))
      .unwrap_or_else(|| "unset".to_string()),
    controls.max_iterations,
    controls.fallback.as_str()
  ));
  input.push_str("STANDARD_IR:\n");
  input.push_str(&target_spec.standard_ir);
  input.push_str("\n\nLLM_IR_SCHEMA_JSON:\n");
  input.push_str(&serde_json::to_string_pretty(compact_schema)?);
  input.push_str("\n\nSCULPT_IR_JSON:\n");
  input.push_str(&serde_json::to_string_pretty(sculpt_ir)?);
  input.push_str("\n\nNONDET_REPORT:\n");
  input.push_str(nondet_report);
  if let Some(prev) = previous_target_ir {
    input.push_str("\n\nPREVIOUS_TARGET_IR:\n");
    input.push_str(&serde_json::to_string_pretty(prev)?);
  }
  Ok(input)
}

fn extract_anthropic_text(value: &Value) -> Option<String> {
  let content = value.get("content")?.as_array()?;
  let mut text = String::new();
  for item in content {
    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
      if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
        text.push_str(t);
      }
    }
  }
  if text.is_empty() { None } else { Some(text) }
}

fn extract_gemini_text(value: &Value) -> Option<String> {
  let candidates = value.get("candidates")?.as_array()?;
  let first = candidates.first()?;
  let content = first.get("content")?;
  let parts = content.get("parts")?.as_array()?;
  let mut text = String::new();
  for part in parts {
    if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
      text.push_str(t);
    }
  }
  if text.is_empty() { None } else { Some(text) }
}

fn extract_openai_token_usage(value: &Value) -> Option<TokenUsage> {
  let usage = value.get("usage")?;
  let input_tokens = usage.get("input_tokens").and_then(Value::as_u64);
  let output_tokens = usage.get("output_tokens").and_then(Value::as_u64);
  let total_tokens = usage.get("total_tokens").and_then(Value::as_u64);
  if input_tokens.is_none() && output_tokens.is_none() && total_tokens.is_none() {
    return None;
  }
  Some(TokenUsage {
    input_tokens,
    output_tokens,
    total_tokens,
  })
}

fn extract_anthropic_token_usage(value: &Value) -> Option<TokenUsage> {
  let usage = value.get("usage")?;
  let input_tokens = usage.get("input_tokens").and_then(Value::as_u64);
  let output_tokens = usage.get("output_tokens").and_then(Value::as_u64);
  let total_tokens = match (input_tokens, output_tokens) {
    (Some(i), Some(o)) => Some(i + o),
    _ => None,
  };
  if input_tokens.is_none() && output_tokens.is_none() {
    return None;
  }
  Some(TokenUsage {
    input_tokens,
    output_tokens,
    total_tokens,
  })
}

fn extract_gemini_token_usage(value: &Value) -> Option<TokenUsage> {
  let usage = value.get("usageMetadata")?;
  let input_tokens = usage.get("promptTokenCount").and_then(Value::as_u64);
  let output_tokens = usage.get("candidatesTokenCount").and_then(Value::as_u64);
  let total_tokens = usage.get("totalTokenCount").and_then(Value::as_u64);
  if input_tokens.is_none() && output_tokens.is_none() && total_tokens.is_none() {
    return None;
  }
  Some(TokenUsage {
    input_tokens,
    output_tokens,
    total_tokens,
  })
}

fn parse_json_response(text: &str) -> Result<Value> {
  let trimmed = text.trim();
  if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
    return Ok(parsed);
  }

  let mut s = trimmed;
  if let Some(stripped) = s.strip_prefix("```") {
    s = stripped.trim_start();
    if let Some(rest) = s.strip_prefix("json") {
      s = rest.trim_start();
    }
    if let Some(end) = s.rfind("```") {
      s = &s[..end];
    }
  }

  if let Ok(parsed) = serde_json::from_str::<Value>(s.trim()) {
    return Ok(parsed);
  }

  if let (Some(start), Some(end)) = (s.find('{'), s.rfind('}')) {
    if start < end {
      let slice = &s[start..=end];
      if let Ok(parsed) = serde_json::from_str::<Value>(slice) {
        return Ok(parsed);
      }
    }
  }

  bail!("Failed to parse JSON from model output")
}

fn patch_target_ir_with_deterministic_parts(target: &mut Value, standard_ir: &str, sculpt_ir: &Value) {
  if standard_ir != "cli-ir" {
    return;
  }
  let Some(root) = target.as_object_mut() else {
    return;
  };
  let Some(flows) = sculpt_ir.get("flows").and_then(Value::as_array) else {
    return;
  };
  let Some(flow) = flows.first() else {
    return;
  };
  let start = flow
    .get("start")
    .and_then(Value::as_str)
    .unwrap_or("Title")
    .to_string();
  let Some(states) = flow.get("states").and_then(Value::as_array) else {
    return;
  };

  let mut transitions = serde_json::Map::new();
  let mut views = serde_json::Map::new();

  for state in states {
    let Some(name) = state.get("name").and_then(Value::as_str) else {
      continue;
    };
    let statements = state
      .get("statements")
      .and_then(Value::as_array)
      .cloned()
      .unwrap_or_default();
    let mut event_map = serde_json::Map::new();
    let mut render_items = Vec::new();

    for stmt in statements {
      if let Some(on) = stmt.get("On").and_then(Value::as_object) {
        let event = on.get("event").and_then(Value::as_object);
        let target_state = on.get("target").and_then(Value::as_str);
        if let (Some(event_obj), Some(dst)) = (event, target_state) {
          let ev_name = event_obj
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
          let ev = normalize_event_name(ev_name, event_obj.get("args").and_then(Value::as_array));
          if !ev.is_empty() {
            event_map.insert(ev, Value::String(dst.to_string()));
          }
        }
      }
      if let Some(expr) = stmt.get("Expr").and_then(Value::as_object) {
        if expr.get("name").and_then(Value::as_str) != Some("render") {
          continue;
        }
        if let Some(args) = expr.get("args").and_then(Value::as_array) {
          if let Some(first) = args.first().and_then(Value::as_object) {
            if let Some(call) = first
              .get("value")
              .and_then(Value::as_object)
              .and_then(|v| v.get("Call"))
              .and_then(Value::as_object)
            {
              let kind = call.get("name").and_then(Value::as_str).unwrap_or_default();
              if kind == "text" || kind == "button" {
                let mut item = serde_json::Map::new();
                item.insert("kind".to_string(), Value::String(kind.to_string()));
                if let Some(call_args) = call.get("args").and_then(Value::as_array) {
                  for (idx, arg) in call_args.iter().enumerate() {
                    let name = arg.get("name").and_then(Value::as_str);
                    let val = arg.get("value");
                    if idx == 0 {
                      if let Some(s) = extract_scalar_string(val) {
                        item.insert("text".to_string(), Value::String(s));
                      }
                    }
                    if name == Some("color") {
                      if let Some(s) = extract_scalar_string(val) {
                        item.insert("color".to_string(), Value::String(s.to_lowercase()));
                      }
                    }
                    if name == Some("style") {
                      if let Some(s) = extract_scalar_string(val) {
                        item.insert("style".to_string(), Value::String(s));
                      }
                    }
                  }
                }
                render_items.push(Value::Object(item));
              }
            }
          }
        }
      }
    }

    transitions.insert(name.to_string(), Value::Object(event_map));
    if !render_items.is_empty() {
      views.insert(name.to_string(), Value::Array(render_items));
    }
  }

  root.insert(
    "flow".to_string(),
    json!({
      "start": start,
      "transitions": transitions
    }),
  );
  root.insert("views".to_string(), Value::Object(views));

  if let Some(state_obj) = build_runtime_state(sculpt_ir) {
    root.insert("state".to_string(), Value::Object(state_obj));
  }
  inject_runtime_rules(root, sculpt_ir);
}

fn build_runtime_state(sculpt_ir: &Value) -> Option<serde_json::Map<String, Value>> {
  let mut state_obj = serde_json::Map::new();
  let global = sculpt_ir.get("global_state").and_then(Value::as_array)?;
  for stmt in global {
    let Some(assign) = stmt.get("Assign").and_then(Value::as_object) else {
      continue;
    };
    let Some(target) = assign.get("target").and_then(Value::as_str) else {
      continue;
    };
    let op = assign.get("op").and_then(Value::as_str).unwrap_or("Set");
    let value = assign.get("value");
    if op == "Set" {
      if let Some(v) = extract_simple_expr(value) {
        state_obj.insert(target.to_string(), v);
      }
    }
  }
  if state_obj.is_empty() {
    None
  } else {
    Some(state_obj)
  }
}

fn inject_runtime_rules(root: &mut serde_json::Map<String, Value>, sculpt_ir: &Value) {
  let Some(rules) = sculpt_ir.get("rules").and_then(Value::as_array) else {
    return;
  };
  let mut runtime_rules = Vec::new();
  for rule in rules {
    let Some(rule_obj) = rule.as_object() else {
      continue;
    };
    let Some(trigger) = rule_obj.get("trigger").and_then(Value::as_object) else {
      continue;
    };
    let event = trigger
      .get("On")
      .and_then(Value::as_object)
      .map(normalize_event_name_from_call)
      .unwrap_or_default();
    let when = trigger
      .get("When")
      .and_then(extract_when_condition);
    if event.is_empty() && when.is_none() {
      continue;
    }
    let scope_flow = rule_obj
      .get("scope_flow")
      .and_then(Value::as_str)
      .map(|s| s.to_string());
    let scope_state = rule_obj
      .get("scope_state")
      .and_then(Value::as_str)
      .map(|s| s.to_string());
    let mut emits = Vec::<Value>::new();
    let mut assigns = Vec::<Value>::new();
    if let Some(body) = rule_obj.get("body").and_then(Value::as_array) {
      for stmt in body {
        if let Some(emit) = stmt.get("Emit").and_then(Value::as_object) {
          if let Some(ev) = emit.get("event").and_then(Value::as_str) {
            emits.push(Value::String(ev.to_string()));
          }
        } else if let Some(assign) = stmt.get("Assign").and_then(Value::as_object) {
          if let Some(target) = assign.get("target").and_then(Value::as_str) {
            let op = assign.get("op").and_then(Value::as_str).unwrap_or("Set");
            if let Some(value) = extract_simple_expr(assign.get("value")) {
              assigns.push(json!({
                "target": target,
                "op": if op == "Add" { "add" } else { "set" },
                "value": value
              }));
            }
          }
        }
      }
    }
    runtime_rules.push(json!({
      "name": rule_obj.get("name").and_then(Value::as_str).unwrap_or("rule"),
      "scopeFlow": scope_flow,
      "scopeState": scope_state,
      "on": if event.is_empty() { Value::Null } else { Value::String(event) },
      "when": when,
      "emit": emits,
      "assign": assigns
    }));
  }

  if runtime_rules.is_empty() {
    return;
  }

  let extensions = root
    .entry("extensions".to_string())
    .or_insert_with(|| Value::Object(serde_json::Map::new()));
  let Some(ext_obj) = extensions.as_object_mut() else {
    return;
  };
  ext_obj.insert("runtimeRules".to_string(), Value::Array(runtime_rules));
}

fn normalize_event_name_from_call(call: &serde_json::Map<String, Value>) -> String {
  let name = call.get("name").and_then(Value::as_str).unwrap_or_default();
  if name == "key" {
    let key = call
      .get("args")
      .and_then(Value::as_array)
      .and_then(|args| args.first())
      .and_then(|arg| extract_scalar_string(arg.get("value")))
      .unwrap_or_default()
      .to_lowercase();
    return format!("key({key})");
  }
  if call
    .get("args")
    .and_then(Value::as_array)
    .map(|a| a.is_empty())
    .unwrap_or(true)
  {
    return name.to_string();
  }
  name.to_string()
}

fn extract_when_condition(value: &Value) -> Option<Value> {
  let obj = value.as_object()?;
  let binary = obj.get("Binary")?.as_object()?;
  let op = binary.get("op").and_then(Value::as_str)?;
  match op {
    "And" | "Or" => {
      let left = extract_when_condition(binary.get("left")?)?;
      let right = extract_when_condition(binary.get("right")?)?;
      Some(json!({
        "kind": "logic",
        "op": if op == "And" { "and" } else { "or" },
        "left": left,
        "right": right
      }))
    }
    "Gte" | "Gt" | "Lt" | "Eq" | "Neq" => {
      let left_ident = binary
        .get("left")
        .and_then(Value::as_object)
        .and_then(|v| v.get("Ident"))
        .and_then(Value::as_str)?;
      let right = extract_simple_expr(binary.get("right"))?;
      let cmp = match op {
        "Gte" => "gte",
        "Gt" => "gt",
        "Lt" => "lt",
        "Eq" => "eq",
        "Neq" => "neq",
        _ => return None,
      };
      Some(json!({
        "kind": "cmp",
        "op": cmp,
        "left": left_ident,
        "right": right
      }))
    }
    _ => None,
  }
}

fn extract_simple_expr(value: Option<&Value>) -> Option<Value> {
  let v = value?;
  if let Some(n) = v.get("Number").and_then(Value::as_f64) {
    return Some(json!(n));
  }
  if let Some(s) = v.get("String").and_then(Value::as_str) {
    return Some(Value::String(s.to_string()));
  }
  if v.get("Null").is_some() {
    return Some(Value::Null);
  }
  if let Some(id) = v.get("Ident").and_then(Value::as_str) {
    return Some(json!({ "ident": id }));
  }
  None
}

fn normalize_event_name(name: &str, args: Option<&Vec<Value>>) -> String {
  if name == "key" {
    let key = args
      .and_then(|list| list.first())
      .and_then(|a| extract_scalar_string(a.get("value")))
      .unwrap_or_default()
      .to_lowercase();
    return format!("key({})", key);
  }
  if let Some(list) = args {
    if list.is_empty() {
      return name.to_string();
    }
  }
  name.to_string()
}

fn extract_scalar_string(value: Option<&Value>) -> Option<String> {
  let v = value?;
  if let Some(s) = v.get("String").and_then(Value::as_str) {
    return Some(s.to_string());
  }
  if let Some(s) = v.get("Ident").and_then(Value::as_str) {
    return Some(s.to_string());
  }
  None
}

fn http_client() -> Result<reqwest::blocking::Client> {
  Ok(
    reqwest::blocking::Client::builder()
      .timeout(Duration::from_secs(120))
      .build()?,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extracts_openai_usage() {
    let value = json!({
      "usage": { "input_tokens": 10, "output_tokens": 15, "total_tokens": 25 }
    });
    let usage = extract_openai_token_usage(&value).expect("usage");
    assert_eq!(usage.input_tokens, Some(10));
    assert_eq!(usage.output_tokens, Some(15));
    assert_eq!(usage.total_tokens, Some(25));
  }

  #[test]
  fn extracts_anthropic_usage_with_computed_total() {
    let value = json!({
      "usage": { "input_tokens": 7, "output_tokens": 9 }
    });
    let usage = extract_anthropic_token_usage(&value).expect("usage");
    assert_eq!(usage.total_tokens, Some(16));
  }

  #[test]
  fn extracts_gemini_usage() {
    let value = json!({
      "usageMetadata": {
        "promptTokenCount": 5,
        "candidatesTokenCount": 6,
        "totalTokenCount": 11
      }
    });
    let usage = extract_gemini_token_usage(&value).expect("usage");
    assert_eq!(usage.input_tokens, Some(5));
    assert_eq!(usage.output_tokens, Some(6));
    assert_eq!(usage.total_tokens, Some(11));
  }
}
