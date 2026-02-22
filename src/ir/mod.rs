use crate::ast;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrModule {
    pub name: String,
    pub namespace: Vec<String>,
    pub fqns: Vec<String>,
    pub meta: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub uses: Vec<ast::UseDecl>,
    #[serde(default)]
    pub imports: Vec<ast::ImportDecl>,
    pub flows: Vec<IrFlow>,
    pub global_state: Vec<ast::StateStmt>,
    pub rules: Vec<ast::Rule>,
    #[serde(default)]
    pub soft_defines: Vec<ast::SoftDefine>,
    pub nd_blocks: Vec<ast::NdBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrFlow {
    pub name: String,
    pub start: Option<String>,
    pub states: Vec<ast::StateBlock>,
}

pub fn from_ast(module: ast::Module) -> IrModule {
    let ast::Module {
        name,
        meta,
        uses,
        imports,
        items,
    } = module;
    let module_name = name.clone();
    let mut flows = Vec::new();
    let mut global_state = Vec::new();
    let mut rules = Vec::new();
    let mut soft_defines = Vec::new();
    let mut nd_blocks = Vec::new();
    let mut fqns = Vec::new();
    fqns.push(module_name.clone());

    for item in items {
        match item {
            ast::Item::Flow(flow) => {
                let flow_fqn = format!("{}.{}", module_name, flow.name);
                fqns.push(flow_fqn.clone());
                for state in &flow.states {
                    if let Some(name) = &state.name {
                        let state_fqn = format!("{flow_fqn}.{}", name);
                        fqns.push(state_fqn.clone());
                        for stmt in &state.statements {
                            if let ast::StateStmt::Rule(rule) = stmt {
                                fqns.push(format!("{state_fqn}.{}", rule.name));
                                rules.push(rule.clone());
                            }
                        }
                    }
                }
                flows.push(IrFlow {
                    name: flow.name,
                    start: flow.start,
                    states: flow.states,
                });
            }
            ast::Item::GlobalState(state) => {
                for stmt in &state.statements {
                    if let ast::StateStmt::Assign { target, .. } = stmt {
                        fqns.push(format!("{}.global.{}", module_name, target));
                    }
                }
                global_state.extend(state.statements);
            }
            ast::Item::Rule(rule) => {
                fqns.push(format!("{}.{}", module_name, rule.name));
                rules.push(rule)
            }
            ast::Item::Define(define) => soft_defines.push(define),
            ast::Item::Nd(nd) => nd_blocks.push(nd),
        }
    }
    fqns.sort();
    fqns.dedup();

    IrModule {
        name: name.clone(),
        namespace: name.split('.').map(|s| s.to_string()).collect(),
        fqns,
        meta,
        uses,
        imports,
        flows,
        global_state,
        rules,
        soft_defines,
        nd_blocks,
    }
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
