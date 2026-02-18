use serde_json::{json, Map, Value};

pub fn compact_schema_for(standard_ir: &str) -> Option<Value> {
  match standard_ir {
    "cli-ir" => Some(schema_base(standard_ir, false, false)),
    "web-ir" => Some(schema_base(standard_ir, false, false)),
    "gui-ir" => Some(schema_base(standard_ir, true, true)),
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

  if let Some(window) = input.get("w") {
    let mut win = Map::new();
    if let Some(arr) = window.as_array() {
      if let Some(title) = arr.get(0).cloned() {
        if !title.is_null() {
          win.insert("title".to_string(), title);
        }
      }
      if let Some(width) = arr.get(1).cloned() {
        if !width.is_null() {
          win.insert("width".to_string(), width);
        }
      }
      if let Some(height) = arr.get(2).cloned() {
        if !height.is_null() {
          win.insert("height".to_string(), height);
        }
      }
    } else if let Some(obj) = window.as_object() {
      if let Some(title) = obj.get("t").cloned() {
        win.insert("title".to_string(), title);
      }
      if let Some(width) = obj.get("w").cloned() {
        win.insert("width".to_string(), width);
      }
      if let Some(height) = obj.get("h").cloned() {
        win.insert("height".to_string(), height);
      }
    }
    if !win.is_empty() {
      out.insert("window".to_string(), Value::Object(win));
    }
  }

  let flow_start = input
    .get("f")
    .and_then(|f| f.as_array())
    .and_then(|arr| arr.get(0))
    .and_then(|v| v.as_str())
    .map(|s| s.to_string());

  if let Some(views) = input.get("u") {
    let mut views_out = Map::new();
    if let Some(arr) = views.as_array() {
      let mut render_items_for_start: Vec<Value> = Vec::new();
      let mut saw_item_list = false;
      for view in arr {
        if let Some(view_arr) = view.as_array() {
          if view_arr.len() >= 2 {
            let name = view_arr[0].as_str().unwrap_or("").to_string();
            let mut list = Vec::new();
            if let Some(items) = view_arr[1].as_array() {
              if let Some(first) = items.get(0) {
                if first.is_array() {
                  saw_item_list = true;
                  for item in items {
                    if let Some(item_arr) = item.as_array() {
                      if let Some(obj) = render_from_array(item_arr) {
                        list.push(Value::Object(obj));
                      }
                    }
                  }
                } else {
                  if let Some(obj) = render_from_array(items) {
                    render_items_for_start.push(Value::Object(obj));
                  }
                }
              }
            }
            if saw_item_list && !name.is_empty() {
              views_out.insert(name, Value::Array(list));
            }
          }
        }
      }
      if !render_items_for_start.is_empty() {
        if let Some(start) = flow_start.clone() {
          views_out.insert(start, Value::Array(render_items_for_start));
        }
      }
    } else if let Some(obj) = views.as_object() {
      for (name, items) in obj {
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
              if let Some(style) = obj.get("st").cloned() {
                item_out.insert("style".to_string(), style);
              }
              if !item_out.is_empty() {
                list.push(Value::Object(item_out));
              }
            }
          }
        }
        views_out.insert(name.clone(), Value::Array(list));
      }
    }
    out.insert("views".to_string(), Value::Object(views_out));
  }

  if let Some(layout) = input.get("l") {
    let mut layout_out = Map::new();
    if let Some(arr) = layout.as_array() {
      for entry in arr {
        if let Some(pair) = entry.as_array() {
          if pair.len() >= 2 {
            let name = pair[0].as_str().unwrap_or("").to_string();
            let mut view_layout = Map::new();
            if let Some(items) = pair[1].as_array() {
              if let Some(padding) = items.get(0) {
                if !padding.is_null() {
                  view_layout.insert("padding".to_string(), padding.clone());
                }
              }
              if let Some(spacing) = items.get(1) {
                if !spacing.is_null() {
                  view_layout.insert("spacing".to_string(), spacing.clone());
                }
              }
              if let Some(align) = items.get(2) {
                if !align.is_null() {
                  view_layout.insert("align".to_string(), align.clone());
                }
              }
              if let Some(background) = items.get(3) {
                if !background.is_null() {
                  view_layout.insert("background".to_string(), background.clone());
                }
              }
            }
            if !name.is_empty() && !view_layout.is_empty() {
              layout_out.insert(name, Value::Object(view_layout));
            }
          }
        }
      }
    }
    if !layout_out.is_empty() {
      out.insert("layout".to_string(), Value::Object(layout_out));
    }
  }

  if let Some(flow) = input.get("f") {
    let mut flow_out = Map::new();
    if let Some(arr) = flow.as_array() {
      if let Some(start) = arr.get(0).cloned() {
        flow_out.insert("start".to_string(), start);
      }
      let mut transitions_out = Map::new();
      if let Some(transitions) = arr.get(1).and_then(|v| v.as_array()) {
        for entry in transitions {
          if let Some(pair) = entry.as_array() {
            if pair.len() >= 2 {
              let from = pair[0].as_str().unwrap_or("").to_string();
              let mut map = Map::new();
              if let Some(items) = pair[1].as_array() {
                for item in items {
                  if let Some(t) = item.as_array() {
                    if t.len() >= 2 {
                      if let (Some(ev), Some(dst)) = (t[0].as_str(), t[1].as_str()) {
                        map.insert(ev.to_string(), Value::String(dst.to_string()));
                      }
                    }
                  }
                }
              }
              if !from.is_empty() {
                transitions_out.insert(from, Value::Object(map));
              }
            }
          }
        }
      }
      flow_out.insert("transitions".to_string(), Value::Object(transitions_out));
    } else if let Some(obj) = flow.as_object() {
      if let Some(start) = obj.get("s").cloned() {
        flow_out.insert("start".to_string(), start);
      }
      if let Some(transitions) = obj.get("t").cloned() {
        flow_out.insert("transitions".to_string(), transitions);
      }
    }
    if !flow_out.is_empty() {
      out.insert("flow".to_string(), Value::Object(flow_out));
    }
  }

  Value::Object(out)
}

fn render_from_array(item_arr: &[Value]) -> Option<Map<String, Value>> {
  let mut item_out = Map::new();
  if let Some(kind) = item_arr.get(0) {
    if !kind.is_null() {
      item_out.insert("kind".to_string(), kind.clone());
    }
  }
  if let Some(text) = item_arr.get(1) {
    if !text.is_null() {
      item_out.insert("text".to_string(), text.clone());
    }
  }
  if let Some(color) = item_arr.get(2) {
    if !color.is_null() {
      item_out.insert("color".to_string(), color.clone());
    }
  }
  if let Some(x) = item_arr.get(3) {
    if !x.is_null() {
      item_out.insert("x".to_string(), x.clone());
    }
  }
  if let Some(y) = item_arr.get(4) {
    if !y.is_null() {
      item_out.insert("y".to_string(), y.clone());
    }
  }
  if let Some(action) = item_arr.get(5) {
    if !action.is_null() {
      item_out.insert("action".to_string(), action.clone());
    }
  }
  if let Some(style) = item_arr.get(6) {
    if !style.is_null() {
      item_out.insert("style".to_string(), style.clone());
    }
  }
  if item_out.is_empty() { None } else { Some(item_out) }
}

fn schema_base(standard_ir: &str, include_window: bool, allow_button: bool) -> Value {
  let mut props = serde_json::Map::new();
  props.insert("t".to_string(), json!({ "const": standard_ir }));
  props.insert("v".to_string(), json!({ "type": "integer", "minimum": 1 }));
  props.insert("s".to_string(), json!({ "type": "object" }));
  props.insert("x".to_string(), json!({ "type": "object" }));

  if include_window {
    props.insert("w".to_string(), json!({
      "type": "array",
      "prefixItems": [
        { "type": "string" },
        { "type": "integer" },
        { "type": "integer" }
      ],
      "items": false
    }));
  }

  let kind_enum = if allow_button {
    json!(["text", "button"])
  } else {
    json!(["text"])
  };

  props.insert("u".to_string(), json!({
    "type": "array",
    "minItems": 1,
    "items": {
      "type": "array",
      "prefixItems": [
        { "type": "string" },
        {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "array",
            "prefixItems": [
              { "enum": kind_enum },
              { "type": ["string", "null"] },
              { "type": ["string", "null"] },
              { "type": ["integer", "null"] },
              { "type": ["integer", "null"] },
              { "type": ["string", "null"] },
              { "type": ["string", "null"] }
            ],
            "items": false
          }
        }
      ],
      "items": false
    }
  }));

  props.insert("f".to_string(), json!({
    "type": "array",
    "prefixItems": [
      { "type": "string" },
      {
        "type": "array",
        "items": {
          "type": "array",
          "prefixItems": [
            { "type": "string" },
            {
              "type": "array",
              "items": {
                "type": "array",
                "prefixItems": [
                  { "type": "string" },
                  { "type": "string" }
                ],
                "items": false
              }
            }
          ],
          "items": false
        }
      }
    ],
    "items": false
  }));

  props.insert("l".to_string(), json!({
    "type": "array",
    "items": {
      "type": "array",
      "prefixItems": [
        { "type": "string" },
        {
          "type": "array",
          "prefixItems": [
            { "type": ["integer", "null"] },
            { "type": ["integer", "null"] },
            { "enum": ["leading", "center", "trailing", null] },
            { "enum": ["window", "grouped", "clear", null] }
          ],
          "items": false
        }
      ],
      "items": false
    }
  }));

  json!({
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": format!("{}-llm", standard_ir),
    "type": "object",
    "required": ["t", "v", "u", "f"],
    "properties": props,
    "additionalProperties": false
  })
}
