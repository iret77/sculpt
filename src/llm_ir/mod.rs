use serde_json::{json, Map, Value};

pub fn compact_schema_for(standard_ir: &str) -> Option<Value> {
  match standard_ir {
    "cli-ir" => Some(schema_base(standard_ir, false)),
    "web-ir" => Some(schema_base(standard_ir, false)),
    "gui-ir" => Some(schema_base(standard_ir, true)),
    _ => None,
  }
}

pub fn normalize_llm_ir(standard_ir: &str, input: &Value) -> Value {
  if input.get("type").is_some() {
    return input.clone();
  }

  let mut out = Map::new();
  out.insert("type".to_string(), Value::String(standard_ir.to_string()));

  if let Some(version) = input.get("v").cloned() {
    out.insert("version".to_string(), version);
  } else {
    out.insert("version".to_string(), Value::Number(1.into()));
  }

  if let Some(state) = input.get("s").cloned() {
    out.insert("state".to_string(), state);
  }

  if let Some(extensions) = input.get("x").cloned() {
    out.insert("extensions".to_string(), extensions);
  }

  if let Some(window) = input.get("w").and_then(|v| v.as_object()) {
    let mut win = Map::new();
    if let Some(title) = window.get("t").cloned() {
      win.insert("title".to_string(), title);
    }
    if let Some(width) = window.get("w").cloned() {
      win.insert("width".to_string(), width);
    }
    if let Some(height) = window.get("h").cloned() {
      win.insert("height".to_string(), height);
    }
    out.insert("window".to_string(), Value::Object(win));
  }

  if let Some(views) = input.get("u").and_then(|v| v.as_object()) {
    let mut views_out = Map::new();
    for (name, items) in views {
      let mut list = Vec::new();
      if let Some(arr) = items.as_array() {
        for item in arr {
          if let Some(obj) = item.as_object() {
            let mut item_out = Map::new();
            if let Some(kind) = obj.get("k").cloned() {
              item_out.insert("kind".to_string(), kind);
            }
            if let Some(text) = obj.get("t").cloned() {
              item_out.insert("text".to_string(), text);
            }
            if let Some(color) = obj.get("c").cloned() {
              item_out.insert("color".to_string(), color);
            }
            if let Some(x) = obj.get("x").cloned() {
              item_out.insert("x".to_string(), x);
            }
            if let Some(y) = obj.get("y").cloned() {
              item_out.insert("y".to_string(), y);
            }
            if let Some(action) = obj.get("a").cloned() {
              item_out.insert("action".to_string(), action);
            }
            if !item_out.is_empty() {
              list.push(Value::Object(item_out));
            }
          }
        }
      }
      views_out.insert(name.clone(), Value::Array(list));
    }
    out.insert("views".to_string(), Value::Object(views_out));
  }

  if let Some(flow) = input.get("f").and_then(|v| v.as_object()) {
    let mut flow_out = Map::new();
    if let Some(start) = flow.get("s").cloned() {
      flow_out.insert("start".to_string(), start);
    }
    if let Some(transitions) = flow.get("t").cloned() {
      flow_out.insert("transitions".to_string(), transitions);
    }
    out.insert("flow".to_string(), Value::Object(flow_out));
  }

  Value::Object(out)
}

fn schema_base(standard_ir: &str, include_window: bool) -> Value {
  let mut props = serde_json::Map::new();
  props.insert("t".to_string(), json!({ "const": standard_ir }));
  props.insert("v".to_string(), json!({ "type": "integer", "minimum": 1 }));
  props.insert("s".to_string(), json!({ "type": "object" }));
  props.insert("x".to_string(), json!({ "type": "object" }));

  if include_window {
    props.insert("w".to_string(), json!({
      "type": "object",
      "properties": {
        "t": { "type": "string" },
        "w": { "type": "integer" },
        "h": { "type": "integer" }
      }
    }));
  }

  props.insert("u".to_string(), json!({
    "type": "object",
    "additionalProperties": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "k": { "type": "string" },
          "t": { "type": "string" },
          "c": { "type": "string" },
          "x": { "type": "integer" },
          "y": { "type": "integer" },
          "a": { "type": "string" }
        }
      }
    }
  }));

  props.insert("f".to_string(), json!({
    "type": "object",
    "properties": {
      "s": { "type": "string" },
      "t": {
        "type": "object",
        "additionalProperties": {
          "type": "object",
          "additionalProperties": { "type": "string" }
        }
      }
    }
  }));

  json!({
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": format!("{}-llm", standard_ir),
    "type": "object",
    "required": ["t", "v", "u", "f"],
    "properties": props
  })
}
