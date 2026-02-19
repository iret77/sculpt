use std::collections::{HashMap, HashSet};

use crate::ast::{Call, Expr, Flow, Item, Module, NdBlock, Rule, StateBlock, StateStmt};

#[derive(Debug, Clone)]
pub struct Diagnostic {
  pub code: &'static str,
  pub message: String,
}

impl Diagnostic {
  fn new(code: &'static str, message: impl Into<String>) -> Self {
    Self { code, message: message.into() }
  }
}

pub fn validate_module(module: &Module) -> Vec<Diagnostic> {
  let mut diagnostics = Vec::new();

  validate_module_name(module, &mut diagnostics);

  let flows: Vec<&Flow> = module
    .items
    .iter()
    .filter_map(|item| match item {
      Item::Flow(f) => Some(f),
      _ => None,
    })
    .collect();
  let rules: Vec<&Rule> = module
    .items
    .iter()
    .filter_map(|item| match item {
      Item::Rule(r) => Some(r),
      _ => None,
    })
    .collect();
  let nd_blocks: Vec<&NdBlock> = module
    .items
    .iter()
    .filter_map(|item| match item {
      Item::Nd(nd) => Some(nd),
      _ => None,
    })
    .collect();

  let known_fqns = collect_known_fqns(module, &flows, &rules);
  validate_flows(&flows, &mut diagnostics);
  validate_rules(&rules, &mut diagnostics);
  validate_nd_blocks(&nd_blocks, &mut diagnostics);
  validate_convergence_meta(module, &nd_blocks, &mut diagnostics);
  validate_run_targets(&flows, &mut diagnostics);
  validate_symbol_references(module, &flows, &rules, &known_fqns, &mut diagnostics);
  validate_shadowing(module, &rules, &mut diagnostics);

  diagnostics
}

pub fn format_diagnostics(diags: &[Diagnostic]) -> String {
  diags
    .iter()
    .map(|d| format!("{}: {}", d.code, d.message))
    .collect::<Vec<_>>()
    .join("\n")
}

fn validate_module_name(module: &Module, diagnostics: &mut Vec<Diagnostic>) {
  if module.name.is_empty() {
    diagnostics.push(Diagnostic::new("NS501", "Module namespace is empty"));
    return;
  }
  for segment in module.name.split('.') {
    if !is_valid_ident(segment) {
      diagnostics.push(Diagnostic::new(
        "NS501",
        format!("Invalid namespace segment '{}'", segment),
      ));
    }
  }
}

fn collect_known_fqns(module: &Module, flows: &[&Flow], rules: &[&Rule]) -> HashSet<String> {
  let mut fqns = HashSet::new();
  fqns.insert(module.name.clone());

  for flow in flows {
    let flow_fqn = format!("{}.{}", module.name, flow.name);
    fqns.insert(flow_fqn.clone());
    for state in &flow.states {
      if let Some(state_name) = &state.name {
        fqns.insert(format!("{flow_fqn}.{state_name}"));
      }
    }
  }

  for rule in rules {
    fqns.insert(format!("{}.{}", module.name, rule.name));
  }

  for item in &module.items {
    if let Item::GlobalState(state) = item {
      for stmt in &state.statements {
        if let StateStmt::Assign { target, .. } = stmt {
          fqns.insert(format!("{}.global.{}", module.name, target));
        }
      }
    }
  }

  fqns
}

fn validate_flows(flows: &[&Flow], diagnostics: &mut Vec<Diagnostic>) {
  let mut flow_names = HashSet::new();
  for flow in flows {
    if !flow_names.insert(flow.name.clone()) {
      diagnostics.push(Diagnostic::new("F101", format!("Duplicate flow '{}'", flow.name)));
    }
  }

  for flow in flows {
    if flow.start.is_none() {
      diagnostics.push(Diagnostic::new("F102", format!("Flow '{}' is missing start", flow.name)));
    }

    let named_states: Vec<&StateBlock> = flow.states.iter().filter(|s| s.name.is_some()).collect();
    let mut state_names = HashSet::new();
    for state in &named_states {
      let name = state.name.as_ref().expect("state name exists");
      if !state_names.insert(name.clone()) {
        diagnostics.push(Diagnostic::new(
          "F104",
          format!("Duplicate state '{}' in flow '{}'", name, flow.name),
        ));
      }
    }

    if let Some(start) = &flow.start {
      if !state_names.contains(start) {
        diagnostics.push(Diagnostic::new(
          "F103",
          format!("Unknown start state '{}' in flow '{}'", start, flow.name),
        ));
      }
    }

    for state in &named_states {
      validate_state_transitions(flow, state, &state_names, diagnostics);
    }
  }
}

fn validate_state_transitions(
  flow: &Flow,
  state: &StateBlock,
  known_states: &HashSet<String>,
  diagnostics: &mut Vec<Diagnostic>,
) {
  let state_name = state.name.as_deref().unwrap_or("<unnamed>");
  let mut handlers: HashMap<String, String> = HashMap::new();

  for stmt in &state.statements {
    if let StateStmt::On { event, target } = stmt {
      if !known_states.contains(target) {
        diagnostics.push(Diagnostic::new(
          "F105",
          format!(
            "Unknown transition target '{}' in flow '{}' state '{}'",
            target, flow.name, state_name
          ),
        ));
      }

      let signature = call_signature(event);
      if let Some(existing_target) = handlers.insert(signature.clone(), target.clone()) {
        diagnostics.push(Diagnostic::new(
          "F106",
          format!(
            "Duplicate event handler '{}' in flow '{}' state '{}' (targets '{}' and '{}')",
            signature, flow.name, state_name, existing_target, target
          ),
        ));
      }
    }
  }
}

fn validate_rules(rules: &[&Rule], diagnostics: &mut Vec<Diagnostic>) {
  let mut rule_names = HashSet::new();
  for rule in rules {
    if !rule_names.insert(rule.name.clone()) {
      diagnostics.push(Diagnostic::new("R201", format!("Duplicate rule '{}'", rule.name)));
    }
  }
}

fn validate_nd_blocks(nd_blocks: &[&NdBlock], diagnostics: &mut Vec<Diagnostic>) {
  for nd in nd_blocks {
    if nd.propose.name.is_empty() {
      diagnostics.push(Diagnostic::new("N301", format!("ND '{}' has no propose call", nd.name)));
    }
    if nd.constraints.is_empty() {
      diagnostics.push(Diagnostic::new("N303", format!("ND '{}' has empty satisfy()", nd.name)));
    }
  }
}

fn validate_convergence_meta(module: &Module, nd_blocks: &[&NdBlock], diagnostics: &mut Vec<Diagnostic>) {
  let nd_budget = module.meta.get("nd_budget").map(|v| v.trim().to_string());
  let confidence = module.meta.get("confidence").map(|v| v.trim().to_string());

  if let Some(raw) = nd_budget {
    match raw.parse::<i32>() {
      Ok(value) if (0..=100).contains(&value) => {
        if !nd_blocks.is_empty() && value == 0 {
          diagnostics.push(Diagnostic::new(
            "N305",
            "nd_budget=0 is incompatible with ND blocks; remove ND or increase budget",
          ));
        }
      }
      _ => diagnostics.push(Diagnostic::new(
        "M701",
        format!("Invalid nd_budget '{}': expected integer in range 0..100", raw),
      )),
    }
  }

  if let Some(raw) = confidence {
    match raw.parse::<f64>() {
      Ok(value) if (0.0..=1.0).contains(&value) => {}
      _ => diagnostics.push(Diagnostic::new(
        "M702",
        format!("Invalid confidence '{}': expected number in range 0.0..1.0", raw),
      )),
    }
  }
}

fn validate_run_targets(flows: &[&Flow], diagnostics: &mut Vec<Diagnostic>) {
  let known_flows: HashSet<String> = flows.iter().map(|f| f.name.clone()).collect();
  for flow in flows {
    for state in &flow.states {
      let state_name = state.name.as_deref().unwrap_or("<unnamed>");
      for stmt in &state.statements {
        if let StateStmt::Run { flow: run_target } = stmt {
          if !known_flows.contains(run_target) {
            diagnostics.push(Diagnostic::new(
              "B401",
              format!(
                "Unknown flow '{}' referenced by run in '{}.{}'",
                run_target, flow.name, state_name
              ),
            ));
          }
        }
      }
    }
  }
}

fn validate_symbol_references(
  module: &Module,
  flows: &[&Flow],
  rules: &[&Rule],
  known_fqns: &HashSet<String>,
  diagnostics: &mut Vec<Diagnostic>,
) {
  let mut short_counts: HashMap<String, usize> = HashMap::new();
  for fqn in known_fqns {
    if let Some(short) = fqn.rsplit('.').next() {
      *short_counts.entry(short.to_string()).or_insert(0) += 1;
    }
  }

  let mut check_ident = |ident: &str, context: &str| {
    if ident.contains('.') {
      if !is_valid_qualified_ident(ident) {
        diagnostics.push(Diagnostic::new(
          "NS501",
          format!("Invalid qualified identifier '{}' in {}", ident, context),
        ));
        return;
      }
      let module_prefix = format!("{}.", module.name);
      if !ident.starts_with(&module_prefix) && ident != module.name {
        diagnostics.push(Diagnostic::new(
          "NS504",
          format!(
            "Illegal cross-namespace reference '{}' in {} (missing contract/import)",
            ident, context
          ),
        ));
        return;
      }
      if !known_fqns.contains(ident) {
        diagnostics.push(Diagnostic::new(
          "NS503",
          format!("Unknown qualified reference '{}' in {}", ident, context),
        ));
      }
    } else if short_counts.get(ident).copied().unwrap_or(0) > 1 {
      diagnostics.push(Diagnostic::new(
        "NS506",
        format!("Ambiguous unqualified reference '{}' in {}", ident, context),
      ));
    }
  };

  for flow in flows {
    for state in &flow.states {
      let state_name = state.name.as_deref().unwrap_or("<unnamed>");
      for stmt in &state.statements {
        match stmt {
          StateStmt::On { event, target } => {
            check_ident(target, &format!("state transition {}.{}", flow.name, state_name));
            walk_call_idents(event, &format!("event {}.{}", flow.name, state_name), &mut check_ident);
          }
          StateStmt::Run { flow: target_flow } => {
            check_ident(target_flow, &format!("run {}.{}", flow.name, state_name));
          }
          StateStmt::Assign { target, value, .. } => {
            check_ident(target, &format!("assignment target {}.{}", flow.name, state_name));
            walk_expr_idents(value, &format!("assignment value {}.{}", flow.name, state_name), &mut check_ident);
          }
          StateStmt::Expr(call) => {
            walk_call_idents(call, &format!("expr {}.{}", flow.name, state_name), &mut check_ident);
          }
          StateStmt::Terminate => {}
        }
      }
    }
  }

  for rule in rules {
    match &rule.trigger {
      crate::ast::RuleTrigger::On(call) => walk_call_idents(
        call,
        &format!("rule trigger {}", rule.name),
        &mut check_ident,
      ),
      crate::ast::RuleTrigger::When(expr) => walk_expr_idents(
        expr,
        &format!("rule trigger {}", rule.name),
        &mut check_ident,
      ),
    }
    for stmt in &rule.body {
      match stmt {
        crate::ast::RuleStmt::Assign { target, value, .. } => {
          check_ident(target, &format!("rule assignment target {}", rule.name));
          walk_expr_idents(value, &format!("rule assignment value {}", rule.name), &mut check_ident);
        }
        crate::ast::RuleStmt::Emit { event } => {
          check_ident(event, &format!("rule emit {}", rule.name));
        }
      }
    }
  }
}

fn validate_shadowing(module: &Module, rules: &[&Rule], diagnostics: &mut Vec<Diagnostic>) {
  let strict = module
    .meta
    .get("strict_scopes")
    .or_else(|| module.meta.get("strict"))
    .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
    .unwrap_or(false);
  if !strict {
    return;
  }

  let mut global_vars = HashSet::new();
  for item in &module.items {
    if let Item::GlobalState(state) = item {
      for stmt in &state.statements {
        if let StateStmt::Assign { target, .. } = stmt {
          global_vars.insert(target.clone());
        }
      }
    }
  }

  for rule in rules {
    for param in &rule.params {
      if global_vars.contains(param) {
        diagnostics.push(Diagnostic::new(
          "NS505",
          format!(
            "Rule '{}' parameter '{}' shadows global symbol in strict scope mode",
            rule.name, param
          ),
        ));
      }
    }
  }
}

fn call_signature(call: &Call) -> String {
  let args = call
    .args
    .iter()
    .map(|arg| {
      if let Some(name) = &arg.name {
        format!("{name}:{}", expr_kind(&arg.value))
      } else {
        expr_kind(&arg.value)
      }
    })
    .collect::<Vec<_>>()
    .join(",");
  format!("{}({})", call.name, args)
}

fn expr_kind(expr: &Expr) -> String {
  match expr {
    Expr::Number(n) => format!("number:{n}"),
    Expr::String(s) => format!("string:{s}"),
    Expr::Null => "null".to_string(),
    Expr::Ident(s) => format!("ident:{s}"),
    Expr::Call(c) => format!("call:{}", call_signature(c)),
    Expr::Binary { op, .. } => format!("binary:{op:?}"),
  }
}

fn is_valid_ident(segment: &str) -> bool {
  let mut chars = segment.chars();
  match chars.next() {
    Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
    _ => return false,
  }
  chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_valid_qualified_ident(name: &str) -> bool {
  if name.is_empty() {
    return false;
  }
  name.split('.').all(is_valid_ident)
}

fn walk_call_idents<F>(call: &Call, context: &str, on_ident: &mut F)
where
  F: FnMut(&str, &str),
{
  on_ident(&call.name, context);
  for arg in &call.args {
    if let Some(name) = &arg.name {
      on_ident(name, context);
    }
    walk_expr_idents(&arg.value, context, on_ident);
  }
}

fn walk_expr_idents<F>(expr: &Expr, context: &str, on_ident: &mut F)
where
  F: FnMut(&str, &str),
{
  match expr {
    Expr::Ident(name) => on_ident(name, context),
    Expr::Call(call) => walk_call_idents(call, context, on_ident),
    Expr::Binary { left, right, .. } => {
      walk_expr_idents(left, context, on_ident);
      walk_expr_idents(right, context, on_ident);
    }
    Expr::Number(_) | Expr::String(_) | Expr::Null => {}
  }
}
