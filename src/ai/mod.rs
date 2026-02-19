use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use serde_json::{json, Value};

use crate::build_meta::TokenUsage;
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
) -> Result<(Value, Option<DebugCapture>)> {
  match provider {
    AiProvider::OpenAI { api_key, model } => {
      let (value, debug) = openai_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir, layout_required)?;
      Ok((value, Some(debug)))
    }
    AiProvider::Anthropic { api_key, model } => {
      let (value, debug) = anthropic_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir, layout_required)?;
      Ok((value, Some(debug)))
    }
    AiProvider::Gemini { api_key, model } => {
      let (value, debug) = gemini_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir, layout_required)?;
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
  layout_required: bool,
) -> Result<(Value, DebugCapture)> {
  let compact_schema = compact_schema_for(&target_spec.standard_ir)
    .ok_or_else(|| anyhow::anyhow!("No compact LLM schema for {}", target_spec.standard_ir))?;
  let input = build_prompt(sculpt_ir, target_spec, &compact_schema, nondet_report, previous_target_ir, layout_required)?;

  let body = json!({
    "model": model,
    "input": input,
    "text": {
      "format": {
        "type": "json_schema",
        "name": "target_ir",
        "schema": compact_schema,
        "strict": true
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
  let normalized = normalize_llm_ir(&target_spec.standard_ir, &parsed);
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
) -> Result<(Value, DebugCapture)> {
  let compact_schema = compact_schema_for(&target_spec.standard_ir)
    .ok_or_else(|| anyhow::anyhow!("No compact LLM schema for {}", target_spec.standard_ir))?;
  let input = build_prompt(sculpt_ir, target_spec, &compact_schema, nondet_report, previous_target_ir, layout_required)?;
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
  let normalized = normalize_llm_ir(&target_spec.standard_ir, &parsed);
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
) -> Result<(Value, DebugCapture)> {
  let compact_schema = compact_schema_for(&target_spec.standard_ir)
    .ok_or_else(|| anyhow::anyhow!("No compact LLM schema for {}", target_spec.standard_ir))?;
  let input = build_prompt(sculpt_ir, target_spec, &compact_schema, nondet_report, previous_target_ir, layout_required)?;
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
  let normalized = normalize_llm_ir(&target_spec.standard_ir, &parsed);
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
