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

#[test]
fn rejects_unknown_unqualified_deterministic_call_for_cli() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  flow(Main):
    start > A
    state(A):
      value = totallyUnknownCall("x")
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  state():
    value = null
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C907"));
}

#[test]
fn rejects_wrong_arity_for_deterministic_call_for_cli() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  flow(Main):
    start > A
    state(A):
      value = csvRead("a.csv", "b.csv")
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  state():
    value = null
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C908"));
}

#[test]
fn rejects_invalid_metric_key_signature_for_cli() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  flow(Main):
    start > A
    state(A):
      value = metric(rec, "unknown_metric")
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  state():
    value = 0
    rec = null
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C909"));
    assert!(msg.contains("allowed"));
}

#[test]
fn rejects_empty_sort_key_list_signature_for_cli() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  flow(Main):
    start > A
    state(A):
      value = sortBy(rows, "")
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  state():
    value = null
    rows = null
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C909"));
    assert!(msg.contains("sort key list is empty"));
}

#[test]
fn rejects_unqualified_nd_constraint_magic_word() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  use(cli.guide) as guide
  flow(Main):
    start > A
    state(A):
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  nd(shape):
    propose guide.layoutProfile("compact")
    satisfy(
      playable()
    )
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C910"));
}

#[test]
fn rejects_nd_constraint_unknown_alias() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  flow(Main):
    start > A
    state(A):
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  nd(shape):
    propose ui.text("x")
    satisfy(
      guide.playable()
    )
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C911"));
}

#[test]
fn accepts_namespaced_data_calls_with_valid_signatures() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  use(cli.data) as data
  flow(Main):
    start > A
    state(A):
      rows = data.csvRead(path)
      count = data.rowCount(rows)
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  state():
    rows = null
    count = 0
    path = "invoices.csv"
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    validate_module_against_contract(&ir, "cli", &contract).expect("must pass");
}

#[test]
fn rejects_namespaced_data_call_wrong_arity() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  use(cli.data) as data
  flow(Main):
    start > A
    state(A):
      rows = data.csvRead("a.csv", "b.csv")
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  state():
    rows = null
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C908"));
}

#[test]
fn rejects_build_report_json_field_type_mismatch() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  flow(Main):
    start > A
    state(A):
      report = buildReportJson("bad", 2, 3, 4, 5, 6, 7, 8, 9, 10, 11)
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  state():
    report = null
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C912"));
    assert!(msg.contains("input_stats.invoices"));
}

#[test]
fn rejects_schema_error_message_with_non_identifier_args() {
    let src = r#"@meta target=cli
module(App.Core):
  use(cli.ui)
  use(cli.input) as input
  flow(Main):
    start > A
    state(A):
      error = schemaErrorMessage("a", "b")
      on input.key(esc) > Exit
    end
    state(Exit):
      terminate
    end
  end
  state():
    error = ""
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("cli").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C909"));
    assert!(msg.contains("schemaErrorMessage"));
}

#[test]
fn rejects_non_portable_namespace_when_profile_is_portable() {
    let src = r#"@meta target=cli
@meta profile=portable
module(App.Core):
  use(cli.ui)
  use(cli.guide) as guide
  flow(Main):
    start > A
    state(A):
      ui.text("x")
      on input.key(esc) > Exit
    end
    state(Exit):
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
    assert!(msg.contains("C913"));
}

#[test]
fn rejects_non_portable_symbol_when_profile_is_portable() {
    let src = r#"@meta target=gui
@meta profile=portable
module(App.Core):
  use(gui.input) as input
  flow(Main):
    start > A
    state(A):
      input.click("go")
      on input.key(enter) > Exit
    end
    state(Exit):
      terminate
    end
  end
end
"#;
    let module = parse_source(src).expect("parse");
    let ir = from_ast(module);
    let spec = describe_target("gui").expect("describe");
    let contract = parse_target_contract(&spec).expect("contract");
    let err = validate_module_against_contract(&ir, "gui", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C914"));
}
