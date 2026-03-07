use sculpt::contracts::contract_signature_for_symbol;

#[test]
fn data_symbol_signature_catalog_includes_core_writers() {
    assert_eq!(
        contract_signature_for_symbol("data", "writeJson"),
        Some("data.writeJson(path, jsonObject)")
    );
    assert_eq!(
        contract_signature_for_symbol("data", "writeCsv"),
        Some("data.writeCsv(path, rows)")
    );
}

#[test]
fn unknown_namespace_or_symbol_has_no_catalog_signature() {
    assert_eq!(
        contract_signature_for_symbol("ui", "text"),
        Some("ui.text(content, color?)")
    );
    assert_eq!(
        contract_signature_for_symbol("input", "key"),
        Some("input.key(name)")
    );
    assert_eq!(
        contract_signature_for_symbol("net", "post"),
        Some("net.post(url, body)")
    );
    assert_eq!(
        contract_signature_for_symbol("guide", "highContrast"),
        Some("guide.highContrast()")
    );
    assert_eq!(contract_signature_for_symbol("data", "doesNotExist"), None);
    assert_eq!(contract_signature_for_symbol("unknown", "x"), None);
}
