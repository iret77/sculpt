use sculpt::llm_ir::normalize_llm_ir;
use serde_json::json;

#[test]
fn normalize_cli_positional() {
  let input = json!({
    "t": "cli-ir",
    "v": 1,
    "u": [
      ["Show", [
        ["text", "Hallo", "yellow", null, null, null, "title"],
        ["text", "Welt", "blue", null, null, null, null]
      ]]
    ],
    "f": ["Show", [["Show", []]]]
  });

  let out = normalize_llm_ir("cli-ir", &input);
  assert_eq!(out.get("type").unwrap().as_str().unwrap(), "cli-ir");
  assert_eq!(out.get("version").unwrap().as_i64().unwrap(), 1);
  let views = out.get("views").unwrap();
  let show = views.get("Show").unwrap().as_array().unwrap();
  assert_eq!(show.len(), 2);
  assert_eq!(show[0].get("kind").unwrap().as_str().unwrap(), "text");
  assert_eq!(show[0].get("text").unwrap().as_str().unwrap(), "Hallo");
  assert_eq!(show[0].get("style").unwrap().as_str().unwrap(), "title");
}

#[test]
fn normalize_gui_positional_with_window() {
  let input = json!({
    "t": "gui-ir",
    "v": 1,
    "w": ["NativeWindow", 400, 150],
    "u": [
      ["Main", [
        ["text", "SCULPT Native Demo", "yellow", 10, 20, null, "title"],
        ["button", "Open OK", null, 10, 80, "press_ok", null]
      ]]
    ],
    "f": ["Main", [["Main", []]]]
  });

  let out = normalize_llm_ir("gui-ir", &input);
  let window = out.get("window").unwrap();
  assert_eq!(window.get("title").unwrap().as_str().unwrap(), "NativeWindow");
  assert_eq!(window.get("width").unwrap().as_i64().unwrap(), 400);
  assert_eq!(window.get("height").unwrap().as_i64().unwrap(), 150);
  let views = out.get("views").unwrap();
  let main = views.get("Main").unwrap().as_array().unwrap();
  assert_eq!(main[1].get("kind").unwrap().as_str().unwrap(), "button");
  assert_eq!(main[1].get("action").unwrap().as_str().unwrap(), "press_ok");
}
