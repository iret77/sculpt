use sculpt::contracts::{parse_target_contract, validate_module_against_contract};
use sculpt::ir::from_ast;
use sculpt::parser::parse_source;
use sculpt::targets::describe_target;

#[test]
fn contract_version_meta_accepts_matching_version() {
    let source = r#"@meta target=cli
@meta contract_version=1
module(App.Core):
  flow(Main):
    start > Exit
    state(Exit):
      terminate
    end
  end
end
"#;
    let module = parse_source(source).expect("parse");
    let ir = from_ast(module);
    let target_spec = describe_target("cli").expect("target spec");
    let contract = parse_target_contract(&target_spec).expect("contract");
    validate_module_against_contract(&ir, "cli", &contract).expect("must validate");
}

#[test]
fn contract_version_meta_rejects_mismatch() {
    let source = r#"@meta target=cli
@meta contract_version=999
module(App.Core):
  flow(Main):
    start > Exit
    state(Exit):
      terminate
    end
  end
end
"#;
    let module = parse_source(source).expect("parse");
    let ir = from_ast(module);
    let target_spec = describe_target("cli").expect("target spec");
    let contract = parse_target_contract(&target_spec).expect("contract");
    let err = validate_module_against_contract(&ir, "cli", &contract).expect_err("must fail");
    let msg = format!("{err}");
    assert!(msg.contains("C915"));
    assert!(msg.contains("contract_version"));
}
