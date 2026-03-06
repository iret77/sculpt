use sculpt::targets::describe_target;

fn package_exports(spec: &serde_json::Value, namespace: &str) -> Vec<String> {
    let packages = spec
        .get("contract")
        .and_then(|c| c.get("packages"))
        .and_then(|p| p.as_array())
        .expect("packages array");
    let pkg = packages
        .iter()
        .find(|p| {
            p.get("namespace")
                .and_then(|n| n.as_str())
                .map(|n| n == namespace)
                .unwrap_or(false)
        })
        .unwrap_or_else(|| panic!("missing package namespace '{}'", namespace));
    pkg.get("exports")
        .and_then(|e| e.as_array())
        .expect("exports array")
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

fn assert_exports_contain(spec: &serde_json::Value, namespace: &str, required: &[&str]) {
    let exports = package_exports(spec, namespace);
    for symbol in required {
        assert!(
            exports.iter().any(|e| e == symbol),
            "namespace '{}' missing required export '{}'",
            namespace,
            symbol
        );
    }
}

#[test]
fn cli_contract_has_practical_core_exports() {
    let spec = describe_target("cli").expect("describe cli");
    assert_exports_contain(&spec, "ui", &["text", "table", "progress", "status"]);
    assert_exports_contain(&spec, "input", &["key", "tick", "submit", "resize"]);
    assert_exports_contain(
        &spec,
        "data",
        &["csvRead", "rowCount", "writeJson", "writeCsv"],
    );
    assert_exports_contain(
        &spec,
        "guide",
        &["playable", "compactTerminalLayout", "highContrast"],
    );
}

#[test]
fn gui_contract_has_practical_core_exports() {
    let spec = describe_target("gui").expect("describe gui");
    assert_exports_contain(&spec, "ui", &["text", "button", "input", "table", "tabs"]);
    assert_exports_contain(&spec, "input", &["key", "click", "submit", "closeWindow"]);
    assert_exports_contain(
        &spec,
        "data",
        &["csvRead", "rowCount", "writeJson", "writeCsv"],
    );
    assert_exports_contain(
        &spec,
        "window",
        &["open", "close", "resize", "modalConfirm", "notify"],
    );
    assert_exports_contain(
        &spec,
        "guide",
        &["desktopNativeLook", "focusOrderStable", "dialogCopyClarity"],
    );
}

#[test]
fn web_contract_has_practical_core_exports() {
    let spec = describe_target("web").expect("describe web");
    assert_exports_contain(&spec, "ui", &["heading", "panel", "table", "tabs", "modal"]);
    assert_exports_contain(&spec, "input", &["key", "submit", "navigate", "refresh"]);
    assert_exports_contain(&spec, "data", &["query", "group", "aggregate", "join"]);
    assert_exports_contain(&spec, "net", &["get", "post", "patch", "upload"]);
    assert_exports_contain(
        &spec,
        "guide",
        &[
            "noOverlap",
            "responsiveBreakpoints",
            "accessibleColorContrast",
        ],
    );
}
