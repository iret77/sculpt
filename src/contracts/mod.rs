use std::collections::{BTreeMap, HashSet};

use anyhow::{bail, Result};
use serde_json::Value;

use crate::ir::IrModule;

#[derive(Debug, Clone)]
enum MetaType {
  Bool,
  IntRange { min: i64, max: i64 },
  FloatRange { min: f64, max: f64 },
  Enum { values: HashSet<String> },
  CapabilityList,
  String,
}

#[derive(Debug, Clone)]
struct MetaFieldSpec {
  key: String,
  meta_type: MetaType,
}

#[derive(Debug, Clone)]
pub struct TargetContract {
  pub version: u32,
  pub capabilities: HashSet<String>,
  meta_schema: BTreeMap<String, MetaFieldSpec>,
}

pub fn parse_target_contract(spec: &Value) -> Result<TargetContract> {
  let contract = spec.get("contract").cloned().unwrap_or_else(|| Value::Object(serde_json::Map::new()));
  let version = contract.get("version").and_then(Value::as_u64).unwrap_or(1) as u32;

  let mut capabilities = HashSet::new();
  if let Some(items) = contract.get("capabilities").and_then(Value::as_array) {
    for item in items {
      if let Some(s) = item.as_str() {
        capabilities.insert(s.to_string());
      }
    }
  }

  let mut meta_schema = default_meta_schema();
  if let Some(meta) = contract.get("meta").and_then(Value::as_object) {
    for (key, value) in meta {
      if let Some(parsed) = parse_meta_field(key, value)? {
        meta_schema.insert(key.to_string(), parsed);
      }
    }
  }

  Ok(TargetContract { version, capabilities, meta_schema })
}

pub fn validate_module_against_contract(ir: &IrModule, target: &str, contract: &TargetContract) -> Result<()> {
  let mut errors = Vec::new();

  for (key, value) in &ir.meta {
    let Some(spec) = contract.meta_schema.get(key) else {
      if !key.starts_with("x_") {
        errors.push(format!(
          "C903: Unknown @meta key '{}' for target '{}' (declare it in target contract meta schema)",
          key, target
        ));
      }
      continue;
    };
    validate_meta_value(spec, value, target, &mut errors);
  }

  if let Some(raw) = ir.meta.get("requires") {
    for capability in parse_capability_list(raw) {
      if !contract.capabilities.contains(&capability) {
        errors.push(format!(
          "C902: Required capability '{}' is not provided by target '{}'",
          capability, target
        ));
      }
    }
  }

  if let Some(layout) = ir.meta.get("layout") {
    if layout.trim().eq_ignore_ascii_case("explicit")
      && !contract.capabilities.contains("layout.explicit")
    {
      errors.push(format!(
        "C904: layout=explicit requires capability 'layout.explicit' on target '{}'",
        target
      ));
    }
  }

  if !errors.is_empty() {
    bail!("{}", errors.join("\n"));
  }
  Ok(())
}

fn default_meta_schema() -> BTreeMap<String, MetaFieldSpec> {
  let mut map = BTreeMap::new();
  map.insert(
    "target".to_string(),
    MetaFieldSpec {
      key: "target".to_string(),
      meta_type: MetaType::String,
    },
  );
  map.insert(
    "nd_budget".to_string(),
    MetaFieldSpec {
      key: "nd_budget".to_string(),
      meta_type: MetaType::IntRange { min: 0, max: 100 },
    },
  );
  map.insert(
    "confidence".to_string(),
    MetaFieldSpec {
      key: "confidence".to_string(),
      meta_type: MetaType::FloatRange { min: 0.0, max: 1.0 },
    },
  );
  map.insert(
    "strict_scopes".to_string(),
    MetaFieldSpec {
      key: "strict_scopes".to_string(),
      meta_type: MetaType::Bool,
    },
  );
  map.insert(
    "requires".to_string(),
    MetaFieldSpec {
      key: "requires".to_string(),
      meta_type: MetaType::CapabilityList,
    },
  );
  map.insert(
    "max_iterations".to_string(),
    MetaFieldSpec {
      key: "max_iterations".to_string(),
      meta_type: MetaType::IntRange { min: 1, max: 10_000 },
    },
  );
  map.insert(
    "fallback".to_string(),
    MetaFieldSpec {
      key: "fallback".to_string(),
      meta_type: MetaType::Enum {
        values: ["fail", "stub", "replay"].iter().map(|s| s.to_string()).collect(),
      },
    },
  );
  map
}

fn parse_meta_field(key: &str, value: &Value) -> Result<Option<MetaFieldSpec>> {
  let Some(obj) = value.as_object() else {
    return Ok(None);
  };
  let kind = obj.get("type").and_then(Value::as_str).unwrap_or("string");
  let meta_type = match kind {
    "bool" => MetaType::Bool,
    "int" => {
      let min = obj.get("min").and_then(Value::as_i64).unwrap_or(i64::MIN);
      let max = obj.get("max").and_then(Value::as_i64).unwrap_or(i64::MAX);
      MetaType::IntRange { min, max }
    }
    "float" => {
      let min = obj.get("min").and_then(Value::as_f64).unwrap_or(f64::MIN);
      let max = obj.get("max").and_then(Value::as_f64).unwrap_or(f64::MAX);
      MetaType::FloatRange { min, max }
    }
    "enum" => {
      let Some(values) = obj.get("values").and_then(Value::as_array) else {
        bail!("Invalid contract meta '{}': enum requires values[]", key);
      };
      let mut set = HashSet::new();
      for v in values {
        if let Some(s) = v.as_str() {
          set.insert(s.to_string());
        }
      }
      MetaType::Enum { values: set }
    }
    "capability_list" => MetaType::CapabilityList,
    _ => MetaType::String,
  };
  Ok(Some(MetaFieldSpec { key: key.to_string(), meta_type }))
}

fn validate_meta_value(spec: &MetaFieldSpec, value: &str, target: &str, errors: &mut Vec<String>) {
  let trimmed = value.trim();
  match &spec.meta_type {
    MetaType::Bool => {
      let lower = trimmed.to_ascii_lowercase();
      let ok = matches!(lower.as_str(), "1" | "0" | "true" | "false" | "yes" | "no" | "on" | "off");
      if !ok {
        errors.push(format!(
          "C901: @meta {}='{}' is invalid for target '{}' (expected bool)",
          spec.key, value, target
        ));
      }
    }
    MetaType::IntRange { min, max } => match trimmed.parse::<i64>() {
      Ok(v) if v >= *min && v <= *max => {}
      _ => errors.push(format!(
        "C901: @meta {}='{}' is invalid for target '{}' (expected int {}..{})",
        spec.key, value, target, min, max
      )),
    },
    MetaType::FloatRange { min, max } => match trimmed.parse::<f64>() {
      Ok(v) if v >= *min && v <= *max => {}
      _ => errors.push(format!(
        "C901: @meta {}='{}' is invalid for target '{}' (expected float {}..{})",
        spec.key, value, target, min, max
      )),
    },
    MetaType::Enum { values } => {
      if !values.contains(trimmed) {
        let mut list: Vec<_> = values.iter().cloned().collect();
        list.sort();
        errors.push(format!(
          "C901: @meta {}='{}' is invalid for target '{}' (expected one of: {})",
          spec.key,
          value,
          target,
          list.join(", ")
        ));
      }
    }
    MetaType::CapabilityList => {}
    MetaType::String => {}
  }
}

fn parse_capability_list(raw: &str) -> Vec<String> {
  raw
    .split(',')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect()
}

