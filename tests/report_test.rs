use sculpt::ir::from_ast;
use sculpt::parser::parse_source;
use sculpt::report::generate_report;

#[test]
fn report_includes_summary_and_budget_status() {
    let src = r#"@meta nd_budget=30
@meta confidence=0.85
@meta max_iterations=3
@meta fallback=stub
module(App):
  nd(layout):
    propose grid()
    satisfy(
      insideBounds(width: 10, height: 5),
      noOverlap()
    )
  end
end
"#;

    let module = parse_source(src).expect("parse ok");
    let ir = from_ast(module);
    let report = generate_report(&ir);
    assert!(report.contains("Convergence Report"));
    assert!(report.contains("nd_budget: 30"));
    assert!(report.contains("confidence: 0.85"));
    assert!(report.contains("max_iterations: 3"));
    assert!(report.contains("fallback: stub"));
    assert!(report.contains("overall_nd_score:"));
    assert!(report.contains("overall_budget_status:"));
}
