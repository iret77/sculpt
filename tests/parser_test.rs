use sculpt::ast::{Item, RuleTrigger, StateStmt};
use sculpt::freeze::compute_ir_hash;
use sculpt::ir::from_ast;
use sculpt::parser::parse_source;

#[test]
fn parses_minimal_module() {
    let src = r#"module(Mini):
    flow(App):
      start > Title
      state(Title):
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
    let src = r#"flow(App):
  end
"#;
    assert!(parse_source(src).is_err());
}

#[test]
fn parses_meta_headers() {
    let src = r#"
@meta target=gui layout=explicit
@meta author="test"
module(App):
end
"#;
    let module = parse_source(src).expect("parse ok");
    assert_eq!(module.meta.get("target").unwrap(), "gui");
    assert_eq!(module.meta.get("layout").unwrap(), "explicit");
    assert_eq!(module.meta.get("author").unwrap(), "test");
}

#[test]
fn missing_end_fails() {
    let src = r#"module(Mini):
  flow(App):
    start > Title
  end
"#;
    assert!(parse_source(src).is_err());
}

#[test]
fn ir_hash_deterministic() {
    let src = r#"module(Mini):
  state():
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
    let src = r#"module(SnakeHighND):
  nd(designSnake):
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
    let src = r#"module(Billing.Account.Invoice):
end
"#;
    let module = parse_source(src).expect("parse ok");
    assert_eq!(module.name, "Billing.Account.Invoice");
}

#[test]
fn parses_use_declarations() {
    let src = r#"module(App):
  use(cli.ui)
  use(cli.input) as input
end
"#;
    let module = parse_source(src).expect("parse ok");
    assert_eq!(module.uses.len(), 2);
    assert_eq!(module.uses[0].path, "cli.ui");
    assert_eq!(module.uses[0].alias.as_deref(), None);
    assert_eq!(module.uses[1].path, "cli.input");
    assert_eq!(module.uses[1].alias.as_deref(), Some("input"));
}

#[test]
fn parses_import_declarations() {
    let src = r#"module(App):
  import(shared.helpers)
  import(shared.ui) as SharedUI
end
"#;
    let module = parse_source(src).expect("parse ok");
    assert_eq!(module.imports.len(), 2);
    assert_eq!(module.imports[0].path, "shared.helpers");
    assert_eq!(module.imports[0].alias.as_deref(), None);
    assert_eq!(module.imports[1].path, "shared.ui");
    assert_eq!(module.imports[1].alias.as_deref(), Some("SharedUI"));
}

#[test]
fn parses_module_and_nd_defines() {
    let src = r#"module(App):
  define collision.stable():
    "Collision should feel stable."
  end
  nd(design):
    define board.size(width, height):
      "Board should be {width}x{height}."
    end
    propose game("snake")
    satisfy(
      playable(),
      ?collision.stable(),
      ?board.size(width: 20, height: 30)
    )
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let defines = module
        .items
        .iter()
        .filter(|i| matches!(i, Item::Define(_)))
        .count();
    assert_eq!(defines, 1);
    let nd = module
        .items
        .iter()
        .find_map(|i| match i {
            Item::Nd(nd) => Some(nd),
            _ => None,
        })
        .expect("nd");
    assert_eq!(nd.defines.len(), 1);
    assert!(nd.constraints.iter().any(|c| c.name == "?collision.stable"));
}

#[test]
fn missing_block_colon_fails() {
    let src = r#"module(App)
end
"#;
    assert!(parse_source(src).is_err());
}

#[test]
fn parses_semicolon_short_form() {
    let src = r#"module(App): flow(Main): start > A; state(A): terminate; end; end; end"#;
    let module = parse_source(src).expect("parse ok");
    assert_eq!(module.name, "App");
}

#[test]
fn parses_state_local_rules_and_on_shortcuts() {
    let src = r#"module(App):
  flow(Main):
    start > Play
    state(Play):
      on key(Left):: paddleX += 1
      on tick:
        emit done
      end
      rule(localTick):
        on tick:: counter += 1
      end
      on done > Play
    end
  end
end
"#;
    let module = parse_source(src).expect("parse ok");
    let flow = module
        .items
        .iter()
        .find_map(|item| match item {
            Item::Flow(f) => Some(f),
            _ => None,
        })
        .expect("flow");
    let state = flow.states.first().expect("state");
    let local_rules = state
        .statements
        .iter()
        .filter_map(|s| match s {
            StateStmt::Rule(r) => Some(r),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(local_rules.len(), 3);
    assert!(local_rules[0].name.starts_with("__on_"));
    assert_eq!(local_rules[0].scope_flow.as_deref(), Some("Main"));
    assert_eq!(local_rules[0].scope_state.as_deref(), Some("Play"));
    assert!(matches!(local_rules[1].trigger, RuleTrigger::On(_)));
    assert_eq!(local_rules[2].name, "localTick");
}

#[test]
fn parses_when_operator_variants() {
    let src = r#"module(App):
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
    assert_eq!(module.name, "App");
}
