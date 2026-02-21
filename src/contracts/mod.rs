use std::collections::{BTreeMap, HashSet};

use anyhow::{bail, Result};
use serde_json::Value;

use crate::ast::{Call, Expr, Rule, RuleTrigger, StateStmt};
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
    packages: BTreeMap<String, ContractPackage>,
}

#[derive(Debug, Clone)]
struct ContractPackage {
    id: String,
    namespace: String,
    exports: HashSet<String>,
}

pub fn parse_target_contract(spec: &Value) -> Result<TargetContract> {
    let contract = spec
        .get("contract")
        .cloned()
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
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

    let mut packages = BTreeMap::new();
    if let Some(items) = contract.get("packages").and_then(Value::as_array) {
        for item in items {
            let Some(obj) = item.as_object() else {
                continue;
            };
            let Some(namespace) = obj.get("namespace").and_then(Value::as_str) else {
                continue;
            };
            let id = obj
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or(namespace)
                .to_string();
            let mut exports = HashSet::new();
            if let Some(list) = obj.get("exports").and_then(Value::as_array) {
                for sym in list {
                    if let Some(s) = sym.as_str() {
                        exports.insert(s.to_string());
                    }
                }
            }
            packages.insert(
                namespace.to_string(),
                ContractPackage {
                    id,
                    namespace: namespace.to_string(),
                    exports,
                },
            );
        }
    }

    Ok(TargetContract {
        version,
        capabilities,
        meta_schema,
        packages,
    })
}

pub fn validate_module_against_contract(
    ir: &IrModule,
    target: &str,
    contract: &TargetContract,
) -> Result<()> {
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

    validate_symbols_against_packages(ir, contract, target, &mut errors);

    if !errors.is_empty() {
        bail!("{}", errors.join("\n"));
    }
    Ok(())
}

fn validate_symbols_against_packages(
    ir: &IrModule,
    contract: &TargetContract,
    target: &str,
    errors: &mut Vec<String>,
) {
    if contract.packages.is_empty() {
        return;
    }

    let mut alias_to_namespace: BTreeMap<String, String> = BTreeMap::new();
    for use_decl in &ir.uses {
        let namespace = use_decl
            .path
            .rsplit('.')
            .next()
            .unwrap_or_default()
            .to_string();
        let alias = use_decl.alias.clone().unwrap_or_else(|| namespace.clone());
        let Some(pkg) = contract.packages.get(&namespace) else {
            errors.push(format!(
                "C905: Unknown package namespace '{}' in use({}) for target '{}'",
                namespace, use_decl.path, target
            ));
            continue;
        };
        let _ = &pkg.namespace;
        alias_to_namespace.insert(alias, namespace);
    }

    let mut check_call = |call: &Call, ctx: &str| {
        let Some((root, symbol)) = split_qualified_call(&call.name) else {
            return;
        };
        let Some(namespace) = alias_to_namespace.get(root) else {
            return;
        };
        let Some(pkg) = contract.packages.get(namespace) else {
            return;
        };
        if !pkg.exports.contains(symbol) {
            let mut exports: Vec<_> = pkg.exports.iter().cloned().collect();
            exports.sort();
            errors.push(format!(
        "C906: Symbol '{}.{}' not exported by package '{}' (target '{}', context: {}, exports: {})",
        root,
        symbol,
        pkg.id,
        target,
        ctx,
        exports.join(", ")
      ));
        }
    };

    for flow in &ir.flows {
        for state in &flow.states {
            let state_name = state.name.as_deref().unwrap_or("<unnamed>");
            for stmt in &state.statements {
                match stmt {
                    StateStmt::On { event, .. } => check_call(
                        event,
                        &format!("flow '{}', state '{}', transition", flow.name, state_name),
                    ),
                    StateStmt::Expr(call) => check_call(
                        call,
                        &format!("flow '{}', state '{}', expression", flow.name, state_name),
                    ),
                    StateStmt::Assign { value, .. } => walk_expr_calls(
                        value,
                        &mut check_call,
                        &format!("flow '{}', state '{}', assignment", flow.name, state_name),
                    ),
                    StateStmt::Rule(rule) => {
                        validate_rule_calls(rule, &mut check_call, flow.name.as_str(), state_name)
                    }
                    StateStmt::Run { .. } | StateStmt::Terminate => {}
                }
            }
        }
    }

    for rule in &ir.rules {
        validate_rule_calls(rule, &mut check_call, "<module>", "<module>");
    }

    for nd in &ir.nd_blocks {
        check_call(&nd.propose, &format!("nd '{}' propose", nd.name));
        for c in &nd.constraints {
            check_call(c, &format!("nd '{}' satisfy", nd.name));
        }
    }
}

fn validate_rule_calls<F>(rule: &Rule, check_call: &mut F, flow_name: &str, state_name: &str)
where
    F: FnMut(&Call, &str),
{
    match &rule.trigger {
        RuleTrigger::On(call) => check_call(
            call,
            &format!(
                "flow '{}', state '{}', rule '{}', trigger",
                flow_name, state_name, rule.name
            ),
        ),
        RuleTrigger::When(expr) => walk_expr_calls(
            expr,
            check_call,
            &format!(
                "flow '{}', state '{}', rule '{}', when",
                flow_name, state_name, rule.name
            ),
        ),
    }
    for stmt in &rule.body {
        if let crate::ast::RuleStmt::Assign { value, .. } = stmt {
            walk_expr_calls(
                value,
                check_call,
                &format!(
                    "flow '{}', state '{}', rule '{}', body",
                    flow_name, state_name, rule.name
                ),
            );
        }
    }
}

fn walk_expr_calls<F>(expr: &Expr, check_call: &mut F, ctx: &str)
where
    F: FnMut(&Call, &str),
{
    match expr {
        Expr::Call(call) => check_call(call, ctx),
        Expr::Binary { left, right, .. } => {
            walk_expr_calls(left, check_call, ctx);
            walk_expr_calls(right, check_call, ctx);
        }
        Expr::Number(_) | Expr::String(_) | Expr::Null | Expr::Ident(_) => {}
    }
}

fn split_qualified_call(name: &str) -> Option<(&str, &str)> {
    let mut parts = name.split('.');
    let root = parts.next()?;
    let symbol = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((root, symbol))
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
            meta_type: MetaType::IntRange {
                min: 1,
                max: 10_000,
            },
        },
    );
    map.insert(
        "fallback".to_string(),
        MetaFieldSpec {
            key: "fallback".to_string(),
            meta_type: MetaType::Enum {
                values: ["fail", "stub", "replay"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
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
    Ok(Some(MetaFieldSpec {
        key: key.to_string(),
        meta_type,
    }))
}

fn validate_meta_value(spec: &MetaFieldSpec, value: &str, target: &str, errors: &mut Vec<String>) {
    let trimmed = value.trim();
    match &spec.meta_type {
        MetaType::Bool => {
            let lower = trimmed.to_ascii_lowercase();
            let ok = matches!(
                lower.as_str(),
                "1" | "0" | "true" | "false" | "yes" | "no" | "on" | "off"
            );
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
    raw.split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}
