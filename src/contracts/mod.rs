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

    for nd in &ir.nd_blocks {
        for c in &nd.constraints {
            if c.name == "?prompt" || c.name.starts_with('?') {
                continue;
            }
            let Some((root, symbol)) = split_qualified_call(&c.name) else {
                errors.push(format!(
                    "C910: ND constraint '{}' in nd '{}' must be explicit: use '?define(...)', '?\"...\"', or a namespaced contract call (e.g. guide.{})",
                    c.name, nd.name, c.name
                ));
                continue;
            };
            let Some(namespace) = alias_to_namespace.get(root) else {
                errors.push(format!(
                    "C911: ND constraint '{}.{}' in nd '{}' uses unknown alias '{}' (import the package with use(...))",
                    root, symbol, nd.name, root
                ));
                continue;
            };
            let Some(pkg) = contract.packages.get(namespace) else {
                continue;
            };
            if !pkg.exports.contains(symbol) {
                let mut exports: Vec<_> = pkg.exports.iter().cloned().collect();
                exports.sort();
                errors.push(format!(
                    "C906: Symbol '{}.{}' not exported by package '{}' (target '{}', context: nd '{}' satisfy, exports: {})",
                    root,
                    symbol,
                    pkg.id,
                    target,
                    nd.name,
                    exports.join(", ")
                ));
            }
        }
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
                    StateStmt::On { .. } => {}
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

    validate_deterministic_runtime_calls(ir, target, errors);
}

fn validate_deterministic_runtime_calls(ir: &IrModule, target: &str, errors: &mut Vec<String>) {
    if target != "cli" {
        return;
    }

    let mut check_call = |call: &Call, ctx: &str| {
        if split_qualified_call(&call.name).is_some() {
            return;
        }

        let Some(expected_arity) = cli_runtime_call_arity(&call.name) else {
            errors.push(format!(
                "C907: Unknown unqualified deterministic call '{}' (target '{}', context: {})",
                call.name, target, ctx
            ));
            return;
        };

        if call.args.len() != expected_arity {
            errors.push(format!(
                "C908: Invalid arg count for deterministic call '{}' (expected {}, got {}, target '{}', context: {})",
                call.name,
                expected_arity,
                call.args.len(),
                target,
                ctx
            ));
            return;
        }

        validate_cli_runtime_signature(call, target, ctx, errors);
    };

    for flow in &ir.flows {
        for state in &flow.states {
            let state_name = state.name.as_deref().unwrap_or("<unnamed>");
            for stmt in &state.statements {
                match stmt {
                    StateStmt::On { .. } => {}
                    StateStmt::Expr(call) => check_call(
                        call,
                        &format!("flow '{}', state '{}', expression", flow.name, state_name),
                    ),
                    StateStmt::Assign { value, .. } => walk_expr_calls(
                        value,
                        &mut check_call,
                        &format!("flow '{}', state '{}', assignment", flow.name, state_name),
                    ),
                    StateStmt::Rule(rule) => validate_rule_expr_calls(
                        rule,
                        &mut check_call,
                        flow.name.as_str(),
                        state_name,
                    ),
                    StateStmt::Run { .. } | StateStmt::Terminate => {}
                }
            }
        }
    }

    for rule in &ir.rules {
        validate_rule_expr_calls(rule, &mut check_call, "<module>", "<module>");
    }
}

fn validate_cli_runtime_signature(call: &Call, target: &str, ctx: &str, errors: &mut Vec<String>) {
    let arg_expr = |idx: usize| call.args.get(idx).map(|a| &a.value);
    let literal_string = |idx: usize| match arg_expr(idx) {
        Some(Expr::String(s)) => Some(s.as_str()),
        _ => None,
    };

    match call.name.as_str() {
        "csvRead" => {
            if !is_path_like_expr(arg_expr(0)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg1 must be path-like string or identifier, target '{}', context: {})",
          call.name, target, ctx
        ));
            }
        }
        "csvHasColumns" => {
            if let Some(columns) = literal_string(1) {
                let cols: Vec<_> = columns
                    .split(',')
                    .map(|c| c.trim())
                    .filter(|c| !c.is_empty())
                    .collect();
                if cols.is_empty() {
                    errors.push(format!(
            "C909: Invalid signature for '{}' (arg2 column list is empty, target '{}', context: {})",
            call.name, target, ctx
          ));
                }
            } else if !is_identifier_expr(arg_expr(1)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg2 must be comma-separated string or identifier, target '{}', context: {})",
          call.name, target, ctx
        ));
            }
        }
        "csvMissingColumns" => {
            if let Some(columns) = literal_string(1) {
                let cols: Vec<_> = columns
                    .split(',')
                    .map(|c| c.trim())
                    .filter(|c| !c.is_empty())
                    .collect();
                if cols.is_empty() {
                    errors.push(format!(
            "C909: Invalid signature for '{}' (arg2 column list is empty, target '{}', context: {})",
            call.name, target, ctx
          ));
                }
            } else if !is_identifier_expr(arg_expr(1)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg2 must be comma-separated string or identifier, target '{}', context: {})",
          call.name, target, ctx
        ));
            }
        }
        "schemaErrorMessage" => {
            if !is_identifier_expr(arg_expr(0)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg1 must be identifier from csvMissingColumns(...), target '{}', context: {})",
          call.name, target, ctx
        ));
            }
            if !is_identifier_expr(arg_expr(1)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg2 must be identifier from csvMissingColumns(...), target '{}', context: {})",
          call.name, target, ctx
        ));
            }
        }
        "metric" => {
            if let Some(metric_name) = literal_string(1) {
                let allowed = [
                    "matched_full",
                    "matched_partial",
                    "overpaid",
                    "missing_payment",
                    "duplicate_payment",
                    "ambiguous",
                    "suspicious",
                ];
                if !allowed.contains(&metric_name) {
                    errors.push(format!(
            "C909: Invalid signature for '{}' (arg2 metric key '{}' not allowed; allowed: {}, target '{}', context: {})",
            call.name,
            metric_name,
            allowed.join(", "),
            target,
            ctx
          ));
                }
            } else if !is_identifier_expr(arg_expr(1)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg2 must be known metric key string or identifier, target '{}', context: {})",
          call.name, target, ctx
        ));
            }
        }
        "sortBy" => {
            if let Some(keys) = literal_string(1) {
                let parsed: Vec<_> = keys
                    .split(',')
                    .map(|c| c.trim())
                    .filter(|c| !c.is_empty())
                    .collect();
                if parsed.is_empty() {
                    errors.push(format!(
            "C909: Invalid signature for '{}' (arg2 sort key list is empty, target '{}', context: {})",
            call.name, target, ctx
          ));
                }
            } else if !is_identifier_expr(arg_expr(1)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg2 must be comma-separated sort-key string or identifier, target '{}', context: {})",
          call.name, target, ctx
        ));
            }
        }
        "writeJson" | "writeCsv" => {
            if !is_path_like_expr(arg_expr(0)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg1 must be output path string or identifier, target '{}', context: {})",
          call.name, target, ctx
        ));
            }
        }
        "summaryLine" => {
            if literal_string(0).is_none() && !is_identifier_expr(arg_expr(0)) {
                errors.push(format!(
          "C909: Invalid signature for '{}' (arg1 must be label string or identifier, target '{}', context: {})",
          call.name, target, ctx
        ));
            }
        }
        "buildReportJson" => {
            let fields = [
                ("input_stats.invoices", ArgType::NumberLike),
                ("input_stats.payments", ArgType::NumberLike),
                ("classification_counts.matched_full", ArgType::NumberLike),
                ("classification_counts.matched_partial", ArgType::NumberLike),
                ("classification_counts.overpaid", ArgType::NumberLike),
                ("classification_counts.missing_payment", ArgType::NumberLike),
                (
                    "classification_counts.duplicate_payment",
                    ArgType::NumberLike,
                ),
                ("classification_counts.ambiguous", ArgType::NumberLike),
                ("classification_counts.suspicious", ArgType::NumberLike),
                ("rules_version", ArgType::StringLike),
                ("processing_ms", ArgType::NumberLike),
            ];
            for (idx, (field_name, expected)) in fields.iter().enumerate() {
                if let Some(expr) = arg_expr(idx) {
                    match validate_arg_type(expr, *expected) {
                        Some(msg) => errors.push(format!(
                            "C912: buildReportJson field '{}' has invalid value ({}, target '{}', context: {})",
                            field_name, msg, target, ctx
                        )),
                        None => {}
                    }
                }
            }
        }
        _ => {}
    }
}

#[derive(Clone, Copy)]
enum ArgType {
    NumberLike,
    StringLike,
}

fn validate_arg_type(expr: &Expr, expected: ArgType) -> Option<&'static str> {
    if matches!(expr, Expr::Null) {
        return Some("null is not allowed");
    }
    match expected {
        ArgType::NumberLike => match expr {
            Expr::Number(_) | Expr::Ident(_) | Expr::Call(_) | Expr::Binary { .. } => None,
            Expr::String(_) => Some("expected numeric expression, got string"),
            Expr::Null => Some("null is not allowed"),
        },
        ArgType::StringLike => match expr {
            Expr::String(_) | Expr::Ident(_) => None,
            Expr::Number(_) => Some("expected string expression, got number"),
            Expr::Call(_) | Expr::Binary { .. } => {
                Some("expected string/identifier, got computed expression")
            }
            Expr::Null => Some("null is not allowed"),
        },
    }
}

fn is_identifier_expr(expr: Option<&Expr>) -> bool {
    matches!(expr, Some(Expr::Ident(_)))
}

fn is_path_like_expr(expr: Option<&Expr>) -> bool {
    matches!(expr, Some(Expr::String(_)) | Some(Expr::Ident(_)))
}

fn cli_runtime_call_arity(name: &str) -> Option<usize> {
    match name {
        "csvRead" => Some(1),
        "rowCount" => Some(1),
        "csvHasColumns" => Some(2),
        "csvMissingColumns" => Some(2),
        "schemaErrorMessage" => Some(2),
        "reconcileInvoices" => Some(4),
        "metric" => Some(2),
        "buildExceptions" => Some(1),
        "buildReportJson" => Some(11),
        "processingMs" => Some(1),
        "writeJson" => Some(2),
        "sortBy" => Some(2),
        "writeCsv" => Some(2),
        "summaryLine" => Some(2),
        _ => None,
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

fn validate_rule_expr_calls<F>(rule: &Rule, check_call: &mut F, flow_name: &str, state_name: &str)
where
    F: FnMut(&Call, &str),
{
    if let RuleTrigger::When(expr) = &rule.trigger {
        walk_expr_calls(
            expr,
            check_call,
            &format!(
                "flow '{}', state '{}', rule '{}', when",
                flow_name, state_name, rule.name
            ),
        );
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
        "required_outputs".to_string(),
        MetaFieldSpec {
            key: "required_outputs".to_string(),
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
    map.insert(
        "nd_critical_path".to_string(),
        MetaFieldSpec {
            key: "nd_critical_path".to_string(),
            meta_type: MetaType::Enum {
                values: ["off", "warn", "error"]
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
