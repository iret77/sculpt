use crate::ast::{Call, Expr};
use crate::ir::IrModule;

pub fn generate_report(ir: &IrModule) -> String {
    let mut out = String::new();
    let budget = ir.meta.get("nd_budget").and_then(|v| v.parse::<i32>().ok());
    let confidence = ir
        .meta
        .get("confidence")
        .and_then(|v| v.parse::<f64>().ok());
    let max_iterations = ir
        .meta
        .get("max_iterations")
        .and_then(|v| v.parse::<u32>().ok());
    let fallback = ir.meta.get("fallback").cloned();
    let mut block_scores: Vec<f64> = Vec::new();

    out.push_str("Convergence Report\n");
    out.push_str("==================\n");
    if let Some(b) = budget {
        out.push_str(&format!("nd_budget: {}\n", b));
    } else {
        out.push_str("nd_budget: (not set)\n");
    }
    if let Some(c) = confidence {
        out.push_str(&format!("confidence: {:.2}\n", c));
    } else {
        out.push_str("confidence: (not set)\n");
    }
    if let Some(max_it) = max_iterations {
        out.push_str(&format!("max_iterations: {}\n", max_it));
    } else {
        out.push_str("max_iterations: (not set)\n");
    }
    if let Some(fb) = fallback {
        out.push_str(&format!("fallback: {}\n", fb));
    } else {
        out.push_str("fallback: (not set)\n");
    }
    out.push('\n');

    for (idx, nd) in ir.nd_blocks.iter().enumerate() {
        let nd_id = format!("{}#{}", nd.name, idx);
        out.push_str(&format!("ND: {} at {}\n", format_call(&nd.propose), nd_id));
        let measurable = nd
            .constraints
            .iter()
            .filter(|c| is_measurable_call(c))
            .count();
        let constraints = nd.constraints.len();
        out.push_str(&format!(
            "constraints: {}, measurable: {}\n",
            constraints, measurable
        ));
        let measurability_ratio = if constraints == 0 {
            0.0
        } else {
            measurable as f64 / constraints as f64
        };
        out.push_str(&format!(
            "measurability_ratio: {:.2}\n",
            measurability_ratio
        ));
        let unconstrained = nd.constraints.is_empty() || measurable == 0;
        out.push_str(&format!(
            "unconstrained: {}\n\n",
            if unconstrained { "yes" } else { "no" }
        ));
        let nd_score = estimate_nd_score(constraints, measurable);
        block_scores.push(nd_score);
        out.push_str(&format!("nd_score: {:.0}/100\n", nd_score));
        out.push_str(&format!("risk: {}\n", classify_risk(nd_score)));
        if let Some(b) = budget {
            let status = if nd_score <= b as f64 {
                "within_budget"
            } else {
                "over_budget"
            };
            out.push_str(&format!(
                "budget_status: {} (budget={}, score={:.0})\n",
                status, b, nd_score
            ));
        }
        out.push('\n');
    }

    let overall_nd = if block_scores.is_empty() {
        0.0
    } else {
        block_scores.iter().sum::<f64>() / block_scores.len() as f64
    };
    out.push_str("Summary\n");
    out.push_str("-------\n");
    out.push_str(&format!("nd_blocks: {}\n", ir.nd_blocks.len()));
    out.push_str(&format!("overall_nd_score: {:.0}/100\n", overall_nd));
    out.push_str(&format!("overall_risk: {}\n", classify_risk(overall_nd)));
    if let Some(b) = budget {
        let status = if overall_nd <= b as f64 {
            "within_budget"
        } else {
            "over_budget"
        };
        out.push_str(&format!("overall_budget_status: {}\n", status));
    }
    out
}

fn estimate_nd_score(constraints: usize, measurable: usize) -> f64 {
    if constraints == 0 {
        return 100.0;
    }
    let meas = measurable as f64 / constraints as f64;
    ((1.0 - meas) * 100.0).clamp(0.0, 100.0)
}

fn classify_risk(score: f64) -> &'static str {
    if score >= 70.0 {
        "high"
    } else if score >= 35.0 {
        "medium"
    } else {
        "low"
    }
}

fn is_measurable_call(call: &Call) -> bool {
    call.args.iter().all(|arg| is_literal(&arg.value))
}

fn is_literal(expr: &Expr) -> bool {
    matches!(expr, Expr::Number(_) | Expr::String(_) | Expr::Null)
}

fn format_call(call: &Call) -> String {
    let args = call
        .args
        .iter()
        .map(|arg| {
            if let Some(name) = &arg.name {
                format!("{}: {}", name, format_expr(&arg.value))
            } else {
                format_expr(&arg.value)
            }
        })
        .collect::<Vec<_>>();
    format!("{}({})", call.name, args.join(", "))
}

fn format_expr(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => n.to_string(),
        Expr::String(s) => format!("\"{}\"", s),
        Expr::Null => "null".to_string(),
        Expr::Ident(s) => s.clone(),
        Expr::Call(c) => format_call(c),
        Expr::Binary { left, op: _, right } => {
            format!("{} >= {}", format_expr(left), format_expr(right))
        }
    }
}
