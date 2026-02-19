use sculpt::parser::parse_source;
use sculpt::semantics::validate_module;

#[test]
fn validates_clean_program() {
  let src = r#"module(Billing.Account.Invoice)
  flow(Main)
    start > Draft
    state(Draft)
      on key(Enter) > Draft
    end
  end

  nd(plan)
    propose layout(type: "basic")
    satisfy(
      valid()
    )
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  let diagnostics = validate_module(&module);
  assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
}

#[test]
fn catches_duplicate_flow_and_missing_start_target() {
  let src = r#"module(App.Core)
  flow(Game)
    start > Missing
    state(Title)
      on key(Enter) > Unknown
      on key(Enter) > Title
      run MissingFlow
    end
  end
  flow(Game)
    start > X
    state(X)
    end
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  let diagnostics = validate_module(&module);
  let codes: Vec<_> = diagnostics.iter().map(|d| d.code).collect();
  assert!(codes.contains(&"F101"));
  assert!(codes.contains(&"F103"));
  assert!(codes.contains(&"F105"));
  assert!(codes.contains(&"F106"));
  assert!(codes.contains(&"B401"));
}

#[test]
fn catches_empty_nd_satisfy() {
  let src = r#"module(App)
  nd(plan)
    propose any()
    satisfy()
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  let diagnostics = validate_module(&module);
  assert!(diagnostics.iter().any(|d| d.code == "N303"));
}

#[test]
fn catches_unknown_and_cross_namespace_qualified_refs() {
  let src = r#"module(Billing.Account.Invoice)
  flow(Main)
    start > Draft
    state(Draft)
      amount = Billing.Account.Invoice.global.total
      customer = External.Domain.User
      on key(Enter) > Billing.Account.Invoice.Main.Done
    end
    state(Done)
      terminate
    end
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  let diagnostics = validate_module(&module);
  assert!(diagnostics.iter().any(|d| d.code == "NS503"));
  assert!(diagnostics.iter().any(|d| d.code == "NS504"));
}

#[test]
fn catches_strict_scope_shadowing() {
  let src = r#"@meta strict_scopes=true
module(App.Core)
  state()
    value = 1
  end

  rule(check, value)
    when value >= 1
      emit done
    end
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  let diagnostics = validate_module(&module);
  assert!(diagnostics.iter().any(|d| d.code == "NS505"));
}

#[test]
fn validates_convergence_meta_ranges() {
  let src = r#"@meta nd_budget=500
@meta confidence=1.5
module(App.Core)
  nd(plan)
    propose any()
    satisfy(valid())
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  let diagnostics = validate_module(&module);
  assert!(diagnostics.iter().any(|d| d.code == "M701"));
  assert!(diagnostics.iter().any(|d| d.code == "M702"));
}

#[test]
fn catches_zero_budget_with_nd() {
  let src = r#"@meta nd_budget=0
module(App.Core)
  nd(plan)
    propose any()
    satisfy(valid())
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  let diagnostics = validate_module(&module);
  assert!(diagnostics.iter().any(|d| d.code == "N305"));
}
