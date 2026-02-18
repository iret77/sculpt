use sculpt::parser::parse_source;
use sculpt::ir::from_ast;
use sculpt::freeze::compute_ir_hash;

#[test]
fn parses_minimal_module() {
  let src = r#"module(Mini)
    flow(App)
      start > Title
      state(Title)
        on key(Enter) > Exit
      end
    end
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  assert_eq!(module.name, "Mini");
}

#[test]
fn missing_module_fails() {
  let src = r#"flow(App)
  end
"#;
  assert!(parse_source(src).is_err());
}

#[test]
fn missing_end_fails() {
  let src = r#"module(Mini)
  flow(App)
    start > Title
  end
"#;
  assert!(parse_source(src).is_err());
}

#[test]
fn ir_hash_deterministic() {
  let src = r#"module(Mini)
  state()
    counter = 0
  end
end
"#;
  let module = parse_source(src).unwrap();
  let ir = from_ast(module);
  let h1 = compute_ir_hash(&ir).unwrap();
  let h2 = compute_ir_hash(&ir).unwrap();
  assert_eq!(h1, h2);
}
