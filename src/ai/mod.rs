use anyhow::{bail, Result};
use serde_json::{json, Value};

pub enum AiProvider {
  OpenAI { api_key: String, model: String },
  Stub,
}

pub struct TargetSpec {
  pub standard_ir: String,
  pub schema: Value,
  pub extensions: Value,
}

pub fn generate_target_ir(
  provider: AiProvider,
  sculpt_ir: &Value,
  target_spec: &TargetSpec,
  nondet_report: &str,
  previous_target_ir: Option<&Value>,
) -> Result<Value> {
  match provider {
    AiProvider::OpenAI { api_key, model } => openai_generate(&api_key, &model, sculpt_ir, target_spec, nondet_report, previous_target_ir),
    AiProvider::Stub => Ok(stub_generate(target_spec)),
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
) -> Result<Value> {
  let mut input = String::new();
  input.push_str("You are the Sculpt compiler AI. Generate target IR JSON that conforms to the provided schema.\n");
  input.push_str("Do not include explanations. Output only JSON.\n\n");
  input.push_str("STANDARD_IR:\n");
  input.push_str(&target_spec.standard_ir);
  input.push_str("\n\nSCULPT_IR_JSON:\n");
  input.push_str(&serde_json::to_string_pretty(sculpt_ir)?);
  input.push_str("\n\nNONDET_REPORT:\n");
  input.push_str(nondet_report);
  if let Some(prev) = previous_target_ir {
    input.push_str("\n\nPREVIOUS_TARGET_IR:\n");
    input.push_str(&serde_json::to_string_pretty(prev)?);
  }

  let body = json!({
    "model": model,
    "input": input,
    "text": {
      "format": {
        "type": "json_schema",
        "json_schema": {
          "name": "target_ir",
          "schema": target_spec.schema,
          "strict": true
        }
      }
    }
  });

  let client = reqwest::blocking::Client::new();
  let resp = client
    .post("https://api.openai.com/v1/responses")
    .bearer_auth(api_key)
    .json(&body)
    .send()?;

  if !resp.status().is_success() {
    bail!("OpenAI error: status {}", resp.status());
  }

  let value: Value = resp.json()?;
  let text = extract_output_text(&value).unwrap_or_default();
  if text.is_empty() {
    bail!("OpenAI returned empty output");
  }

  let parsed: Value = serde_json::from_str(&text)?;
  Ok(parsed)
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
