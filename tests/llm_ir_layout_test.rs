use sculpt::llm_ir::normalize_llm_ir;
use serde_json::json;

#[test]
fn normalize_layout_tokens() {
  let input = json!({
    "t": "gui-ir",
    "v": 1,
    "u": [
      ["Main", [["text", "Title", "yellow", null, null, null, "title"]]]
    ],
    "f": ["Main", [["Main", []]]],
    "l": [
      ["Main", [24, 16, "leading", "window"]]
    ]
  });

  let out = normalize_llm_ir("gui-ir", &input);
  let layout = out.get("layout").unwrap();
  let main = layout.get("Main").unwrap();
  assert_eq!(main.get("padding").unwrap().as_i64().unwrap(), 24);
  assert_eq!(main.get("spacing").unwrap().as_i64().unwrap(), 16);
  assert_eq!(main.get("align").unwrap().as_str().unwrap(), "leading");
  assert_eq!(main.get("background").unwrap().as_str().unwrap(), "window");
}
