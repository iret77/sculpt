use std::time::Instant;

use anyhow::{bail, Result};
use serde_json::{json, Value};

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
}

pub fn generate_target_ir(
  provider: AiProvider,
  sculpt_ir: &Value,
  target_spec: &TargetSpec,
  nondet_report: &str,
  previous_target_ir: Option<&Value>,
) -> Result<(Value, Option<DebugCapture>)> {
  match provider {
    AiProvider::OpenAI { api_key, model } => {
      let (value, debug) = openai_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir)?;
      Ok((value, Some(debug)))
    }
    AiProvider::Anthropic { api_key, model } => {
      let (value, debug) = anthropic_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir)?;
      Ok((value, Some(debug)))
    }
    AiProvider::Gemini { api_key, model } => {
      let (value, debug) = gemini_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir)?;
      Ok((value, Some(debug)))
    }
    AiProvider::Stub => Ok((stub_generate(target_spec), None)),
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
) -> Result<(Value, DebugCapture)> {
  let input = build_prompt(sculpt_ir, target_spec, nondet_report, previous_target_ir)?;

  let body = json!({
    "model": model,
    "input": input,
    "text": {
      "format": {
        "type": "json_schema",
        "name": "target_ir",
        "schema": target_spec.schema,
        "strict": false
      }
    }
  });

  let client = reqwest::blocking::Client::new();
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
  Ok((
    parsed,
    DebugCapture {
      prompt: input,
      raw_output: text,
      llm_ms: started.elapsed().as_millis(),
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
) -> Result<(Value, DebugCapture)> {
  let input = build_prompt(sculpt_ir, target_spec, nondet_report, previous_target_ir)?;
  let body = json!({
    "model": model,
    "max_tokens": 2048,
    "system": "You are the Sculpt compiler AI. Generate target IR JSON that conforms to the provided schema. Output only JSON.",
    "messages": [
      { "role": "user", "content": input }
    ]
  });

  let client = reqwest::blocking::Client::new();
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
  Ok((
    parsed,
    DebugCapture {
      prompt: input,
      raw_output: text,
      llm_ms: started.elapsed().as_millis(),
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
) -> Result<(Value, DebugCapture)> {
  let input = build_prompt(sculpt_ir, target_spec, nondet_report, previous_target_ir)?;
  let body = json!({
    "contents": [
      { "role": "user", "parts": [ { "text": input } ] }
    ],
    "generationConfig": {
      "response_mime_type": "application/json"
    }
  });

  let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model);
  let client = reqwest::blocking::Client::new();
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
  Ok((
    parsed,
    DebugCapture {
      prompt: input,
      raw_output: text,
      llm_ms: started.elapsed().as_millis(),
    },
  ))
}

fn build_prompt(
  sculpt_ir: &Value,
  target_spec: &TargetSpec,
  nondet_report: &str,
  previous_target_ir: Option<&Value>,
) -> Result<String> {
  let mut input = String::new();
  input.push_str("You are the Sculpt compiler AI. Generate target IR JSON that conforms to the provided schema.\n");
  input.push_str("Do not include explanations. Output only JSON.\n\n");
  input.push_str("STANDARD_IR:\n");
  input.push_str(&target_spec.standard_ir);
  input.push_str("\n\nSCHEMA_JSON:\n");
  input.push_str(&serde_json::to_string_pretty(&target_spec.schema)?);
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
