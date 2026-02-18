use crate::ast;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrModule {
  pub name: String,
  pub meta: std::collections::HashMap<String, String>,
  pub flows: Vec<IrFlow>,
  pub global_state: Vec<ast::StateStmt>,
  pub rules: Vec<ast::Rule>,
  pub nd_blocks: Vec<ast::NdBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrFlow {
  pub name: String,
  pub start: Option<String>,
  pub states: Vec<ast::StateBlock>,
}

pub fn from_ast(module: ast::Module) -> IrModule {
  let mut flows = Vec::new();
  let mut global_state = Vec::new();
  let mut rules = Vec::new();
  let mut nd_blocks = Vec::new();

  for item in module.items {
    match item {
      ast::Item::Flow(flow) => {
        flows.push(IrFlow { name: flow.name, start: flow.start, states: flow.states });
      }
      ast::Item::GlobalState(state) => {
        global_state.extend(state.statements);
      }
      ast::Item::Rule(rule) => rules.push(rule),
      ast::Item::Nd(nd) => nd_blocks.push(nd),
    }
  }

  IrModule { name: module.name, meta: module.meta, flows, global_state, rules, nd_blocks }
}

pub fn canonical_json(value: &Value) -> Value {
  match value {
    Value::Object(map) => {
      let mut keys: Vec<_> = map.keys().cloned().collect();
      keys.sort();
      let mut new_map = Map::new();
      for k in keys {
        new_map.insert(k.clone(), canonical_json(&map[&k]));
      }
      Value::Object(new_map)
    }
    Value::Array(items) => Value::Array(items.iter().map(canonical_json).collect()),
    _ => value.clone(),
  }
}

pub fn to_canonical_string(ir: &IrModule) -> Result<String> {
  let value = serde_json::to_value(ir)?;
  let canonical = canonical_json(&value);
  Ok(serde_json::to_string(&canonical)?)
}

pub fn to_pretty_json(ir: &IrModule) -> Result<String> {
  Ok(serde_json::to_string_pretty(ir)?)
}
