use sculpt::parser::parse_source;
use sculpt::ir::from_ast;
use sculpt::freeze::compute_ir_hash;
use sculpt::ast::Item;

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
fn parses_meta_headers() {
  let src = r#"
@meta target=gui layout=explicit
@meta author="test"
module(App)
end
"#;
  let module = parse_source(src).expect("parse ok");
  assert_eq!(module.meta.get("target").unwrap(), "gui");
  assert_eq!(module.meta.get("layout").unwrap(), "explicit");
  assert_eq!(module.meta.get("author").unwrap(), "test");
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

#[test]
fn parses_multiline_satisfy_constraints() {
  let src = r#"module(SnakeHighND)
  nd(designSnake)
    propose game("snake", size: 10)
    satisfy(
      playable(),
      funToLearn(),
      usesKeys("WASD"),
      loopedGameplay()
    )
  end
end
"#;
  let module = parse_source(src).expect("parse ok");
  let nds: Vec<_> = module
    .items
    .iter()
    .filter_map(|i| match i {
      Item::Nd(nd) => Some(nd),
      _ => None,
    })
    .collect();
  assert_eq!(nds.len(), 1);
  assert_eq!(nds[0].constraints.len(), 4);
}

#[test]
fn parses_dot_qualified_module_name() {
  let src = r#"module(Billing.Account.Invoice)
end
"#;
  let module = parse_source(src).expect("parse ok");
  assert_eq!(module.name, "Billing.Account.Invoice");
}
