use std::collections::{HashMap, HashSet};

use crate::ast::{
    BinaryOp, Call, Expr, Flow, Item, Module, NdBlock, Rule, RuleStmt, RuleTrigger, SoftDefine,
    StateBlock, StateStmt,
};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}

impl Diagnostic {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub fn validate_module(module: &Module) -> Vec<Diagnostic> {
    let additional_imported_roots = HashSet::new();
    validate_module_with_imports(module, &additional_imported_roots)
}

pub fn validate_module_with_imports(
    module: &Module,
    additional_imported_roots: &HashSet<String>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    validate_module_name(module, &mut diagnostics);
    let mut imported_roots = validate_use_decls(module, &mut diagnostics);
    validate_import_decls(module, &mut imported_roots, &mut diagnostics);
    imported_roots.extend(additional_imported_roots.iter().cloned());

    let flows: Vec<&Flow> = module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Flow(f) => Some(f),
            _ => None,
        })
        .collect();
    let mut rules: Vec<&Rule> = module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Rule(r) => Some(r),
            _ => None,
        })
        .collect();
    for flow in &flows {
        for state in &flow.states {
            for stmt in &state.statements {
                if let StateStmt::Rule(rule) = stmt {
                    rules.push(rule);
                }
            }
        }
    }
    let nd_blocks: Vec<&NdBlock> = module
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Nd(nd) => Some(nd),
            _ => None,
        })
        .collect();
    let module_defines = collect_module_defines(module, &mut diagnostics);

    let known_fqns = collect_known_fqns(module, &flows, &rules);
    validate_flows(&flows, &mut diagnostics);
    validate_rules(&rules, &mut diagnostics);
    validate_nd_blocks(&nd_blocks, &module_defines, &mut diagnostics);
    validate_convergence_meta(module, &nd_blocks, &mut diagnostics);
    validate_state_execution(&flows, &mut diagnostics);
    validate_legacy_shorthand(module, &flows, &rules, &mut diagnostics);
    validate_symbol_references(
        module,
        &flows,
        &rules,
        &known_fqns,
        &imported_roots,
        &mut diagnostics,
    );
    validate_shadowing(module, &rules, &mut diagnostics);

    diagnostics
}

fn validate_legacy_shorthand(
    _module: &Module,
    flows: &[&Flow],
    rules: &[&Rule],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for flow in flows {
        for state in &flow.states {
            let state_name = state.name.as_deref().unwrap_or("<unnamed>");
            for stmt in &state.statements {
                match stmt {
                    StateStmt::On { event, .. } => {
                        if event.name == "key" {
                            diagnostics.push(Diagnostic::new(
                                "U610",
                                format!(
                                    "Legacy event shorthand 'key(...)' in {}.{}; use 'input.key(...)' with use(...) import",
                                    flow.name, state_name
                                ),
                            ));
                        }
                    }
                    StateStmt::Expr(call) => {
                        if call.name == "render" {
                            diagnostics.push(Diagnostic::new(
                                "U611",
                                format!(
                                    "Legacy render shorthand in {}.{}; use namespaced calls like 'ui.text(...)'",
                                    flow.name, state_name
                                ),
                            ));
                        }
                    }
                    StateStmt::Rule(rule) => {
                        if let RuleTrigger::On(call) = &rule.trigger {
                            if call.name == "key" {
                                diagnostics.push(Diagnostic::new(
                                    "U610",
                                    format!(
                                        "Legacy event shorthand in rule '{}'; use 'input.key(...)' with use(...) import",
                                        rule.name
                                    ),
                                ));
                            }
                        }
                    }
                    StateStmt::Run { .. } | StateStmt::Terminate | StateStmt::Assign { .. } => {}
                }
            }
        }
    }

    for rule in rules {
        if let RuleTrigger::On(call) = &rule.trigger {
            if call.name == "key" {
                diagnostics.push(Diagnostic::new(
                    "U610",
                    format!(
                        "Legacy event shorthand in rule '{}'; use 'input.key(...)' with use(...) import",
                        rule.name
                    ),
                ));
            }
        }
    }
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

fn validate_use_decls(module: &Module, diagnostics: &mut Vec<Diagnostic>) -> HashSet<String> {
    let mut roots = HashSet::new();
    for decl in &module.uses {
        if !is_valid_qualified_ident(&decl.path) {
            diagnostics.push(Diagnostic::new(
                "U601",
                format!("Invalid use path '{}'", decl.path),
            ));
            continue;
        }
        let exposed = decl
            .alias
            .as_ref()
            .cloned()
            .unwrap_or_else(|| decl.path.rsplit('.').next().unwrap_or("").to_string());
        if exposed.is_empty() || !is_valid_ident(&exposed) {
            diagnostics.push(Diagnostic::new(
                "U602",
                format!("Invalid use alias '{}' for path '{}'", exposed, decl.path),
            ));
            continue;
        }
        if !roots.insert(exposed.clone()) {
            diagnostics.push(Diagnostic::new(
                "U603",
                format!("Duplicate imported namespace root '{}'", exposed),
            ));
        }
    }
    roots
}

fn validate_import_decls(
    module: &Module,
    imported_roots: &mut HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for decl in &module.imports {
        let exposed = decl
            .alias
            .as_ref()
            .cloned()
            .unwrap_or_else(|| decl.path.split('.').next().unwrap_or("").trim().to_string());
        if exposed.is_empty() || !is_valid_ident(&exposed) {
            diagnostics.push(Diagnostic::new(
                "U604",
                format!(
                    "Invalid import alias/root '{}' for path '{}'",
                    exposed, decl.path
                ),
            ));
            continue;
        }
        if !imported_roots.insert(exposed.clone()) {
            diagnostics.push(Diagnostic::new(
                "U605",
                format!("Duplicate imported namespace root '{}'", exposed),
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
        if let (Some(flow), Some(state)) = (&rule.scope_flow, &rule.scope_state) {
            fqns.insert(format!("{}.{}.{}.{}", module.name, flow, state, rule.name));
        } else {
            fqns.insert(format!("{}.{}", module.name, rule.name));
        }
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
            diagnostics.push(Diagnostic::new(
                "F101",
                format!("Duplicate flow '{}'", flow.name),
            ));
        }
    }

    for flow in flows {
        if flow.start.is_none() {
            diagnostics.push(Diagnostic::new(
                "F102",
                format!("Flow '{}' is missing start", flow.name),
            ));
        }

        let named_states: Vec<&StateBlock> =
            flow.states.iter().filter(|s| s.name.is_some()).collect();
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
            diagnostics.push(Diagnostic::new(
                "R201",
                format!("Duplicate rule '{}'", rule.name),
            ));
        }
        if rule.body.is_empty() {
            diagnostics.push(Diagnostic::new(
                "R202",
                format!("Rule '{}' has no effect body", rule.name),
            ));
        }
        if let RuleTrigger::When(expr) = &rule.trigger {
            if !is_supported_when_expr(expr) {
                diagnostics.push(Diagnostic::new(
                    "R204",
                    format!(
                        "Rule '{}' uses 'when' without a supported expression (expected comparisons >=, >, <, ==, != and optional and/or)",
                        rule.name
                    ),
                ));
            }
        }
        for stmt in &rule.body {
            if let RuleStmt::Emit { event } = stmt {
                if !is_valid_ident(event) {
                    diagnostics.push(Diagnostic::new(
                        "R205",
                        format!("Rule '{}' emits invalid event name '{}'", rule.name, event),
                    ));
                }
            }
        }
    }
}

fn collect_module_defines(
    module: &Module,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for item in &module.items {
        if let Item::Define(SoftDefine { name, params, .. }) = item {
            if !is_valid_qualified_ident(name) {
                diagnostics.push(Diagnostic::new(
                    "N307",
                    format!("Invalid module define name '{}'", name),
                ));
                continue;
            }
            if out.insert(name.clone(), params.len()).is_some() {
                diagnostics.push(Diagnostic::new(
                    "N307",
                    format!("Duplicate module define '{}'", name),
                ));
            }
        }
    }
    out
}

fn validate_nd_blocks(
    nd_blocks: &[&NdBlock],
    module_defines: &HashMap<String, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for nd in nd_blocks {
        if nd.propose.name.is_empty() {
            diagnostics.push(Diagnostic::new(
                "N301",
                format!("ND '{}' has no propose call", nd.name),
            ));
        }
        if nd.constraints.is_empty() {
            diagnostics.push(Diagnostic::new(
                "N303",
                format!("ND '{}' has empty satisfy()", nd.name),
            ));
        }
        let mut local_defines: HashMap<String, usize> = HashMap::new();
        for d in &nd.defines {
            if !is_valid_qualified_ident(&d.name) {
                diagnostics.push(Diagnostic::new(
                    "N307",
                    format!("ND '{}' has invalid define name '{}'", nd.name, d.name),
                ));
                continue;
            }
            if !local_defines.insert(d.name.clone(), d.params.len()).is_none() {
                diagnostics.push(Diagnostic::new(
                    "N307",
                    format!("ND '{}' has duplicate define '{}'", nd.name, d.name),
                ));
            }
        }
        let mut signatures = HashSet::new();
        for constraint in &nd.constraints {
            let signature = call_signature(constraint);
            if !signatures.insert(signature.clone()) {
                diagnostics.push(Diagnostic::new(
                    "N304",
                    format!(
                        "ND '{}' has duplicate satisfy constraint '{}'",
                        nd.name, signature
                    ),
                ));
            }
            if let Some(raw_name) = constraint.name.strip_prefix('?') {
                let expected_arity = local_defines
                    .get(raw_name)
                    .copied()
                    .or_else(|| module_defines.get(raw_name).copied());
                match expected_arity {
                    Some(expected) => {
                        if expected != constraint.args.len() {
                            diagnostics.push(Diagnostic::new(
                                "N308",
                                format!(
                                    "ND '{}' soft define '?{}' expects {} arg(s), got {}",
                                    nd.name,
                                    raw_name,
                                    expected,
                                    constraint.args.len()
                                ),
                            ));
                        }
                    }
                    None => diagnostics.push(Diagnostic::new(
                        "N309",
                        format!(
                            "ND '{}' references unknown soft define '?{}'",
                            nd.name, raw_name
                        ),
                    )),
                }
            }
        }
    }
}

fn validate_convergence_meta(
    module: &Module,
    nd_blocks: &[&NdBlock],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let nd_policy = module
        .meta
        .get("nd_policy")
        .map(|v| v.trim().to_ascii_lowercase());
    let nd_budget = module.meta.get("nd_budget").map(|v| v.trim().to_string());
    let confidence = module.meta.get("confidence").map(|v| v.trim().to_string());
    let max_iterations = module
        .meta
        .get("max_iterations")
        .map(|v| v.trim().to_string());
    let fallback = module
        .meta
        .get("fallback")
        .map(|v| v.trim().to_ascii_lowercase());

    if let Some(raw) = nd_policy {
        if raw != "strict" {
            diagnostics.push(Diagnostic::new(
                "M705",
                format!("Invalid nd_policy '{}': expected strict", raw),
            ));
        }
    }

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
                format!(
                    "Invalid nd_budget '{}': expected integer in range 0..100",
                    raw
                ),
            )),
        }
    }

    if let Some(raw) = confidence {
        match raw.parse::<f64>() {
            Ok(value) if (0.0..=1.0).contains(&value) => {}
            _ => diagnostics.push(Diagnostic::new(
                "M702",
                format!(
                    "Invalid confidence '{}': expected number in range 0.0..1.0",
                    raw
                ),
            )),
        }
    }

    if let Some(raw) = max_iterations {
        match raw.parse::<u32>() {
            Ok(value) if (1..=10_000).contains(&value) => {}
            _ => diagnostics.push(Diagnostic::new(
                "M703",
                format!(
                    "Invalid max_iterations '{}': expected integer in range 1..10000",
                    raw
                ),
            )),
        }
    }

    if let Some(raw) = fallback {
        if !matches!(raw.as_str(), "fail" | "stub" | "replay") {
            diagnostics.push(Diagnostic::new(
                "M704",
                format!(
                    "Invalid fallback '{}': expected one of fail|stub|replay",
                    raw
                ),
            ));
        }
    }
}

fn validate_state_execution(flows: &[&Flow], diagnostics: &mut Vec<Diagnostic>) {
    let known_flows: HashSet<String> = flows.iter().map(|f| f.name.clone()).collect();
    for flow in flows {
        for state in &flow.states {
            let state_name = state.name.as_deref().unwrap_or("<unnamed>");
            let mut run_targets = Vec::new();
            let mut has_done_handler = false;

            for (idx, stmt) in state.statements.iter().enumerate() {
                if let StateStmt::Terminate = stmt {
                    if idx + 1 != state.statements.len() {
                        diagnostics.push(Diagnostic::new(
                            "B402",
                            format!(
                                "terminate must be the last statement in '{}.{}'",
                                flow.name, state_name
                            ),
                        ));
                    }
                    if state.statements.len() > 1 {
                        diagnostics.push(Diagnostic::new(
                            "B402",
                            format!(
                                "terminate cannot be combined with other statements in '{}.{}'",
                                flow.name, state_name
                            ),
                        ));
                    }
                }
                if let StateStmt::On { event, .. } = stmt {
                    if event.name == "done" {
                        has_done_handler = true;
                    }
                }
            }

            for stmt in &state.statements {
                if let StateStmt::Run { flow: run_target } = stmt {
                    run_targets.push(run_target.clone());
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

            if run_targets.len() > 1 {
                diagnostics.push(Diagnostic::new(
                    "B403",
                    format!(
                        "State '{}.{}' has multiple run targets ({})",
                        flow.name,
                        state_name,
                        run_targets.join(", ")
                    ),
                ));
            }
            if !run_targets.is_empty() && !has_done_handler {
                diagnostics.push(Diagnostic::new(
                    "B404",
                    format!(
                        "State '{}.{}' uses run without an explicit on done > ... transition",
                        flow.name, state_name
                    ),
                ));
            }
        }
    }
}

fn validate_symbol_references(
    module: &Module,
    flows: &[&Flow],
    rules: &[&Rule],
    known_fqns: &HashSet<String>,
    imported_roots: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut short_counts: HashMap<String, usize> = HashMap::new();
    for fqn in known_fqns {
        if let Some(short) = fqn.rsplit('.').next() {
            *short_counts.entry(short.to_string()).or_insert(0) += 1;
        }
    }

    let mut check_ident = |ident: &str, context: &str| {
        if ident.starts_with('?') {
            // Explicit ND-magic identifier (prefixed with '?') is always allowed.
            return;
        }
        if ident.contains('.') {
            if !is_valid_qualified_ident(ident) {
                diagnostics.push(Diagnostic::new(
                    "NS501",
                    format!("Invalid qualified identifier '{}' in {}", ident, context),
                ));
                return;
            }
            if let Some(root) = ident.split('.').next() {
                if imported_roots.contains(root) {
                    return;
                }
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
                        check_ident(
                            target,
                            &format!("state transition {}.{}", flow.name, state_name),
                        );
                        walk_call_idents(
                            event,
                            &format!("event {}.{}", flow.name, state_name),
                            &mut check_ident,
                        );
                    }
                    StateStmt::Run { flow: target_flow } => {
                        check_ident(target_flow, &format!("run {}.{}", flow.name, state_name));
                    }
                    StateStmt::Assign { target, value, .. } => {
                        check_ident(
                            target,
                            &format!("assignment target {}.{}", flow.name, state_name),
                        );
                        walk_expr_idents(
                            value,
                            &format!("assignment value {}.{}", flow.name, state_name),
                            &mut check_ident,
                        );
                    }
                    StateStmt::Expr(call) => {
                        walk_call_idents(
                            call,
                            &format!("expr {}.{}", flow.name, state_name),
                            &mut check_ident,
                        );
                    }
                    StateStmt::Rule(rule) => {
                        match &rule.trigger {
                            RuleTrigger::On(call) => walk_call_idents(
                                call,
                                &format!(
                                    "state rule trigger {}.{}.{}",
                                    flow.name, state_name, rule.name
                                ),
                                &mut check_ident,
                            ),
                            RuleTrigger::When(expr) => walk_expr_idents(
                                expr,
                                &format!(
                                    "state rule trigger {}.{}.{}",
                                    flow.name, state_name, rule.name
                                ),
                                &mut check_ident,
                            ),
                        }
                        for stmt in &rule.body {
                            match stmt {
                                RuleStmt::Assign { target, value, .. } => {
                                    check_ident(
                                        target,
                                        &format!(
                                            "state rule assignment target {}.{}.{}",
                                            flow.name, state_name, rule.name
                                        ),
                                    );
                                    walk_expr_idents(
                                        value,
                                        &format!(
                                            "state rule assignment value {}.{}.{}",
                                            flow.name, state_name, rule.name
                                        ),
                                        &mut check_ident,
                                    );
                                }
                                RuleStmt::Emit { event } => {
                                    check_ident(
                                        event,
                                        &format!(
                                            "state rule emit {}.{}.{}",
                                            flow.name, state_name, rule.name
                                        ),
                                    );
                                }
                            }
                        }
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
                    walk_expr_idents(
                        value,
                        &format!("rule assignment value {}", rule.name),
                        &mut check_ident,
                    );
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

fn is_supported_when_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Binary { op, left, right } => match op {
            BinaryOp::And | BinaryOp::Or => {
                is_supported_when_expr(left) && is_supported_when_expr(right)
            }
            BinaryOp::Gte | BinaryOp::Gt | BinaryOp::Lt | BinaryOp::Eq | BinaryOp::Neq => {
                matches!(**left, Expr::Ident(_))
                    && matches!(
                        **right,
                        Expr::Number(_) | Expr::String(_) | Expr::Null | Expr::Ident(_)
                    )
            }
        },
        _ => false,
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
