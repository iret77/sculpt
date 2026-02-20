use sculpt::parser::parse_source;
use sculpt::semantics::validate_module;

#[test]
fn validates_clean_program() {
    let src = r#"module(Billing.Account.Invoice):
  flow(Main):
    start > Draft
    state(Draft):
      on key(Enter) > Draft
    end
  end

  nd(plan):
    propose layout(type: "basic")
    satisfy(
      valid()
    )
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn catches_duplicate_flow_and_missing_start_target() {
    let src = r#"module(App.Core):
  flow(Game):
    start > Missing
    state(Title):
      on key(Enter) > Unknown
      on key(Enter) > Title
      run MissingFlow
    end
  end
  flow(Game):
    start > X
    state(X):
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
    assert!(codes.contains(&"B404"));
}

#[test]
fn catches_empty_nd_satisfy() {
    let src = r#"module(App):
  nd(plan):
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
    let src = r#"module(Billing.Account.Invoice):
  flow(Main):
    start > Draft
    state(Draft):
      amount = Billing.Account.Invoice.global.total
      customer = External.Domain.User
      on key(Enter) > Billing.Account.Invoice.Main.Done
    end
    state(Done):
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
module(App.Core):
  state():
    value = 1
  end

  rule(check, value):
    when value >= 1:
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
@meta max_iterations=0
@meta fallback=oops
module(App.Core):
  nd(plan):
    propose any()
    satisfy(valid())
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(diagnostics.iter().any(|d| d.code == "M701"));
    assert!(diagnostics.iter().any(|d| d.code == "M702"));
    assert!(diagnostics.iter().any(|d| d.code == "M703"));
    assert!(diagnostics.iter().any(|d| d.code == "M704"));
}

#[test]
fn catches_zero_budget_with_nd() {
    let src = r#"@meta nd_budget=0
module(App.Core):
  nd(plan):
    propose any()
    satisfy(valid())
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(diagnostics.iter().any(|d| d.code == "N305"));
}

#[test]
fn catches_invalid_terminate_placement() {
    let src = r#"module(App.Core):
  flow(Main):
    start > End
    state(End):
      terminate
      on done > End
    end
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(diagnostics.iter().any(|d| d.code == "B402"));
}

#[test]
fn catches_multiple_run_targets_in_state() {
    let src = r#"module(App.Core):
  flow(Main):
    start > Runner
    state(Runner):
      run A
      run B
      on done > Runner
    end
  end

  flow(A):
    start > A1
    state(A1):
      terminate
    end
  end

  flow(B):
    start > B1
    state(B1):
      terminate
    end
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(diagnostics.iter().any(|d| d.code == "B403"));
}

#[test]
fn catches_non_comparison_when_and_invalid_emit_event_name() {
    let src = r#"module(App.Core):
  rule(bad):
    when isReady():
      emit not.valid.event
    end
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(diagnostics.iter().any(|d| d.code == "R204"));
    assert!(diagnostics.iter().any(|d| d.code == "R205"));
}

#[test]
fn catches_duplicate_satisfy_constraints() {
    let src = r#"module(App.Core):
  nd(plan):
    propose any()
    satisfy(
      valid(),
      valid()
    )
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(diagnostics.iter().any(|d| d.code == "N304"));
}

#[test]
fn rejects_nd_magicword_in_strict_policy() {
    let src = r#"@meta nd_policy=strict
module(App.Core):
  flow(Main):
    start > A
    state(A):
      on key(?Escape) > A
    end
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(diagnostics.iter().any(|d| d.code == "N306"));
}

#[test]
fn allows_nd_magicword_in_magic_policy() {
    let src = r#"@meta nd_policy=magic
module(App.Core):
  flow(Main):
    start > A
    state(A):
      on key(?Escape) > A
    end
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(!diagnostics.iter().any(|d| d.code == "N306"));
}

#[test]
fn allows_when_comparison_operators() {
    let src = r#"module(App.Core):
  state():
    score = 0
    limit = 10
    mode = "auto"
  end

  rule(a):
    when score > 0:
      emit done
    end
  end

  rule(b):
    when score < limit:
      emit done
    end
  end

  rule(c):
    when mode == "auto":
      emit done
    end
  end

  rule(d):
    when score != limit and mode == "auto" or score > 2:
      emit done
    end
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let diagnostics = validate_module(&module);
    assert!(!diagnostics.iter().any(|d| d.code == "R204"));
}
