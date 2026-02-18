use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetIr {
  #[serde(rename = "type")]
  pub ir_type: String,
  pub version: u32,
  #[serde(default)]
  pub state: Value,
  pub views: HashMap<String, Vec<RenderItem>>,
  pub flow: Flow,
  #[serde(default)]
  pub window: Option<Window>,
  #[serde(default)]
  pub extensions: Value,
  #[serde(flatten)]
  pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
  pub start: String,
  pub transitions: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderItem {
  pub kind: String,
  pub text: Option<String>,
  pub color: Option<String>,
  pub x: Option<i64>,
  pub y: Option<i64>,
  pub css: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
  pub title: Option<String>,
  pub width: Option<i64>,
  pub height: Option<i64>,
}

pub fn from_json_value(value: Value) -> Result<TargetIr, serde_json::Error> {
  serde_json::from_value(value)
}

pub fn to_json_value(ir: &TargetIr) -> Value {
  serde_json::to_value(ir).unwrap_or_else(|_| Value::Null)
}
