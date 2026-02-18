use crate::ast::{Call, Expr};
use crate::ir::IrModule;

pub fn generate_report(ir: &IrModule) -> String {
  let mut out = String::new();
  for (idx, nd) in ir.nd_blocks.iter().enumerate() {
    let nd_id = format!("{}#{}", nd.name, idx);
    out.push_str(&format!("ND: {} at {}\n", format_call(&nd.propose), nd_id));
    let measurable = nd
      .constraints
      .iter()
      .filter(|c| is_measurable_call(c))
      .count();
    out.push_str(&format!("constraints: {}, measurable: {}\n", nd.constraints.len(), measurable));
    let unconstrained = nd.constraints.is_empty() || measurable == 0;
    out.push_str(&format!("unconstrained: {}\n\n", if unconstrained { "yes" } else { "no" }));
  }
  out
}

fn is_measurable_call(call: &Call) -> bool {
  call.args.iter().all(|arg| is_literal(&arg.value))
}

fn is_literal(expr: &Expr) -> bool {
  matches!(expr, Expr::Number(_) | Expr::String(_) | Expr::Null)
}

fn format_call(call: &Call) -> String {
  let args = call.args.iter().map(|arg| {
    if let Some(name) = &arg.name {
      format!("{}: {}", name, format_expr(&arg.value))
    } else {
      format_expr(&arg.value)
    }
  }).collect::<Vec<_>>();
  format!("{}({})", call.name, args.join(", "))
}

fn format_expr(expr: &Expr) -> String {
  match expr {
    Expr::Number(n) => n.to_string(),
    Expr::String(s) => format!("\"{}\"", s),
    Expr::Null => "null".to_string(),
    Expr::Ident(s) => s.clone(),
    Expr::Call(c) => format_call(c),
    Expr::Binary { left, op: _, right } => format!("{} >= {}", format_expr(left), format_expr(right)),
  }
}
