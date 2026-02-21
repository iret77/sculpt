use sculpt::contracts::{parse_target_contract, validate_module_against_contract};
use sculpt::ir::from_ast;
use sculpt::parser::parse_source;
use sculpt::targets::describe_target;

#[test]
fn rejects_requires_capability_missing_on_target() {
    let src = r#"@meta target=cli
@meta requires="ui.modal.ok"
module(App.Core):
  flow(Main):
    start > A
    state(A):
      terminate
    end
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C902"));
}

#[test]
fn rejects_unknown_meta_key_without_extension_prefix() {
    let src = r#"@meta target=cli
@meta foo=bar
module(App.Core):
  flow(Main):
    start > A
    state(A):
      terminate
    end
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C903"));
}

#[test]
fn allows_layout_explicit_for_gui() {
    let src = r#"@meta target=gui
@meta layout=explicit
module(App.Core):
  flow(Main):
    start > A
    state(A):
      terminate
    end
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("gui").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    validate_module_against_contract(&ir, "gui", &contract).expect("must pass");
}

#[test]
fn rejects_unknown_use_package_namespace_for_target() {
    let src = r#"@meta target=web
module(App.Core):
  use(web.shell)
  flow(Main):
    start > A
    state(A):
      shell.exec("ls")
      terminate
    end
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("web").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "web", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C905"));
}

#[test]
fn rejects_symbol_not_exported_by_package() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  flow(Main):
    start > A
    state(A):
      ui.unknown("x")
      terminate
    end
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C906"));
}
