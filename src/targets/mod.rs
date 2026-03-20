use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use crate::codegen::cli::generate_cli_js;
use crate::codegen::web::generate_web_js;
use crate::ir::{to_pretty_json, IrModule};
use crate::target_ir::TargetIr;

struct GuiViewData {
    window_title: String,
    width: i64,
    height: i64,
}

pub enum TargetKind {
    Cli,
    Web,
    Gui,
    External(String),
}

pub fn resolve_target(name: &str) -> TargetKind {
    match name {
        "cli" => TargetKind::Cli,
        "web" => TargetKind::Web,
        "gui" => TargetKind::Gui,
        other => TargetKind::External(other.to_string()),
    }
}

pub fn run_external_target(
    target: &str,
    ir: &IrModule,
    nd_outputs: Option<&HashMap<String, Value>>,
    target_ir: Option<&Value>,
    out_dir: &Path,
    input_path: &Path,
    lock: Option<Value>,
    mode: &str,
) -> Result<()> {
    let exe = format!("sculpt-target-{}", target);
    let payload = json!({
      "mode": mode,
      "ir": serde_json::to_value(ir)?,
      "irPretty": to_pretty_json(ir)?,
      "ndOutputs": nd_outputs,
      "targetIr": target_ir,
      "outDir": out_dir,
      "input": input_path,
      "lock": lock,
    });

    let mut child = Command::new(&exe)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("Failed to launch target provider: {}", exe))?;

    if let Some(mut stdin) = child.stdin.take() {
        let data = serde_json::to_vec(&payload)?;
        stdin.write_all(&data)?;
    }

    let status = child.wait()?;
    if !status.success() {
        bail!(
            "Target provider {} failed with status {:?}",
            exe,
            status.code()
        );
    }
    Ok(())
}

pub fn emit_web(target: &TargetIr, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(out_dir.join("main.js"), generate_web_js(target))?;
    let html = r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>SCULPT Web Target</title>
    <style>
      :root {
        --bg: #0b111a;
        --bg-elev: #101a29;
        --bg-card: #142131;
        --line: rgba(0, 255, 255, 0.28);
        --text: #eaf5ff;
        --muted: #9fb6c8;
        --accent: #00ffff;
        --accent-2: #ea5172;
        --good: #66f7a8;
        --warn: #ffd166;
      }

      * { box-sizing: border-box; }
      html, body { height: 100%; }
      body {
        margin: 0;
        color: var(--text);
        font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", Inter, Arial, sans-serif;
        background:
          radial-gradient(1200px 700px at 10% -20%, rgba(0, 255, 255, 0.12), transparent 60%),
          radial-gradient(900px 700px at 95% 0%, rgba(234, 81, 114, 0.12), transparent 55%),
          linear-gradient(180deg, #0b111a 0%, #0a0f17 100%);
      }

      #app {
        max-width: 1080px;
        margin: 28px auto;
        padding: 18px;
        border: 1px solid var(--line);
        border-radius: 14px;
        background: linear-gradient(180deg, rgba(15,24,38,0.95), rgba(11,17,26,0.95));
        box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35);
      }

      .sculpt-shell {
        border: 1px solid var(--line);
        border-radius: 12px;
        overflow: hidden;
        background: linear-gradient(180deg, rgba(20,33,49,0.9), rgba(14,24,36,0.9));
      }
      .sculpt-titlebar {
        padding: 10px 14px;
        color: var(--accent);
        font-weight: 700;
        letter-spacing: .03em;
        border-bottom: 1px solid var(--line);
        background: rgba(0, 75, 115, 0.2);
      }
      .sculpt-body {
        padding: 16px;
        display: grid;
        grid-template-columns: 1fr;
        gap: 10px;
      }
      .sculpt-heading { margin: 2px 0 8px; color: var(--accent); font-size: 1.2rem; }
      .sculpt-text { color: var(--text); line-height: 1.4; }
      .sculpt-list { color: var(--text); }
      .sculpt-badge {
        display: inline-block;
        width: fit-content;
        padding: 2px 8px;
        border-radius: 999px;
        border: 1px solid rgba(0,255,255,.35);
        background: rgba(0,255,255,.12);
        color: var(--accent);
        font-size: .86rem;
      }
      .sculpt-metric {
        padding: 8px 10px;
        border: 1px solid var(--line);
        border-radius: 8px;
        background: rgba(0,75,115,.16);
        color: var(--text);
      }
      .sculpt-card {
        padding: 10px;
        border: 1px solid var(--line);
        border-radius: 10px;
        background: rgba(16,26,41,.8);
      }
      .sculpt-tabs {
        color: var(--muted);
        border-bottom: 1px solid rgba(159,182,200,.3);
        padding-bottom: 6px;
      }
      .sculpt-table {
        margin: 0;
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
        color: #cfe4f3;
        background: #0f1724;
        border: 1px solid rgba(159,182,200,.3);
        border-radius: 8px;
        padding: 10px;
        white-space: pre-wrap;
      }
      .sculpt-input {
        width: 100%;
        border: 1px solid rgba(0,255,255,.35);
        border-radius: 8px;
        background: rgba(8,14,22,.9);
        color: var(--text);
        padding: 8px 10px;
      }
      .sculpt-btn {
        width: fit-content;
        border: 1px solid rgba(0,255,255,.4);
        border-radius: 8px;
        padding: 8px 12px;
        background: linear-gradient(180deg, rgba(0,75,115,.36), rgba(0,46,72,.42));
        color: var(--text);
        cursor: pointer;
      }
      .sculpt-btn:hover { border-color: var(--accent); box-shadow: 0 0 0 1px rgba(0,255,255,.2) inset; }
    </style>
  </head>
  <body>
    <div id="app"></div>
    <script src="main.js"></script>
  </body>
</html>
"#;
    std::fs::write(out_dir.join("index.html"), html)?;
    Ok(())
}

pub fn emit_gui(target: &TargetIr, out_dir: &Path) -> Result<()> {
    let data = extract_gui_view_data(target);
    if looks_like_snake_target(target) {
        return emit_gui_tkinter_snake(out_dir, &data);
    }
    match std::env::consts::OS {
        "macos" => emit_gui_macos_swift(target, out_dir, &data),
        _ => emit_gui_tkinter(target, out_dir, &data),
    }
}

fn emit_gui_macos_swift(target: &TargetIr, out_dir: &Path, data: &GuiViewData) -> Result<()> {
    let app_dir = out_dir.join("gui");
    let sources_dir = app_dir.join("Sources");
    std::fs::create_dir_all(&sources_dir)?;

    let package = r#"// swift-tools-version: 5.9
import PackageDescription

let package = Package(
  name: "SculptGui",
  platforms: [.macOS(.v13)],
  products: [
    .executable(name: "SculptGui", targets: ["SculptGui"])
  ],
  targets: [
    .executableTarget(name: "SculptGui", path: "Sources")
  ]
)
"#;
    std::fs::write(app_dir.join("Package.swift"), package)?;

    let target_json = serde_json::to_string(target).unwrap_or_else(|_| "{}".to_string());
    let window_title = &data.window_title;
    let width = data.width;
    let height = data.height;
    let layout = target
        .layout
        .as_ref()
        .and_then(|map| map.get(&target.flow.start));
    let padding = layout.and_then(|l| l.padding).unwrap_or(24);
    let spacing = layout.and_then(|l| l.spacing).unwrap_or(12);
    let align = layout.and_then(|l| l.align.as_deref()).unwrap_or("leading");
    let background = layout
        .and_then(|l| l.background.as_deref())
        .unwrap_or("window");

    let mut swift = String::new();
    swift.push_str("import SwiftUI\nimport AppKit\n\n");
    swift.push_str("struct RenderItem: Codable, Identifiable {\n");
    swift.push_str("  var id = UUID()\n");
    swift.push_str("  let kind: String\n");
    swift.push_str("  let text: String?\n");
    swift.push_str("  let color: String?\n");
    swift.push_str("  let action: String?\n");
    swift.push_str("  let style: String?\n");
    swift.push_str("  enum CodingKeys: String, CodingKey { case kind, text, color, action, style }\n");
    swift.push_str("}\n\n");
    swift.push_str("struct FlowData: Codable {\n");
    swift.push_str("  let start: String\n");
    swift.push_str("  let transitions: [String: [String: String]]\n");
    swift.push_str("}\n\n");
    swift.push_str("struct TargetData: Codable {\n");
    swift.push_str("  let views: [String: [RenderItem]]\n");
    swift.push_str("  let flow: FlowData\n");
    swift.push_str("}\n\n");
    swift.push_str("final class KeyView: NSView {\n");
    swift.push_str("  var onKey: ((String) -> Void)?\n");
    swift.push_str("  override var acceptsFirstResponder: Bool { true }\n");
    swift.push_str("  override func viewDidMoveToWindow() {\n");
    swift.push_str("    super.viewDidMoveToWindow()\n");
    swift.push_str("    DispatchQueue.main.async { self.window?.makeFirstResponder(self) }\n");
    swift.push_str("  }\n");
    swift.push_str("  override func keyDown(with event: NSEvent) {\n");
    swift.push_str("    if let mapped = Self.map(event) { onKey?(mapped) }\n");
    swift.push_str("  }\n");
    swift.push_str("  static func map(_ event: NSEvent) -> String? {\n");
    swift.push_str("    switch event.keyCode {\n");
    swift.push_str("    case 123: return \"left\"\n");
    swift.push_str("    case 124: return \"right\"\n");
    swift.push_str("    case 125: return \"down\"\n");
    swift.push_str("    case 126: return \"up\"\n");
    swift.push_str("    default: break\n");
    swift.push_str("    }\n");
    swift.push_str("    let chars = event.charactersIgnoringModifiers ?? \"\"\n");
    swift.push_str("    if chars == \"\\r\" { return \"enter\" }\n");
    swift.push_str("    if chars == \"\\u{1b}\" { return \"esc\" }\n");
    swift.push_str("    if chars == \" \" { return \"space\" }\n");
    swift.push_str("    let t = chars.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()\n");
    swift.push_str("    return t.isEmpty ? nil : t\n");
    swift.push_str("  }\n");
    swift.push_str("}\n\n");
    swift.push_str("struct KeyCapture: NSViewRepresentable {\n");
    swift.push_str("  let onKey: (String) -> Void\n");
    swift.push_str("  func makeNSView(context: Context) -> KeyView {\n");
    swift.push_str("    let v = KeyView(frame: .zero)\n");
    swift.push_str("    v.onKey = onKey\n");
    swift.push_str("    return v\n");
    swift.push_str("  }\n");
    swift.push_str("  func updateNSView(_ nsView: KeyView, context: Context) {\n");
    swift.push_str("    nsView.onKey = onKey\n");
    swift.push_str("  }\n");
    swift.push_str("}\n\n");
    swift.push_str("final class AppDelegate: NSObject, NSApplicationDelegate {\n");
    swift.push_str("  func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }\n");
    swift.push_str("}\n\n");
    swift.push_str("struct ContentView: View {\n");
    swift.push_str("  let targetData: TargetData\n");
    swift.push_str("  @State private var currentState: String\n");
    swift.push_str("  @State private var showAlert = false\n");
    swift.push_str("  init() {\n");
    swift.push_str("    let json = \"");
    swift.push_str(&escape_swift(&target_json));
    swift.push_str("\"\n");
    swift.push_str("    let decoded = (try? JSONDecoder().decode(TargetData.self, from: Data(json.utf8)))\n");
    swift.push_str("      ?? TargetData(views: [:], flow: FlowData(start: \"\", transitions: [:]))\n");
    swift.push_str("    self.targetData = decoded\n");
    swift.push_str("    _currentState = State(initialValue: decoded.flow.start)\n");
    swift.push_str("  }\n\n");
    swift.push_str("  var activeItems: [RenderItem] { targetData.views[currentState] ?? [] }\n\n");
    swift.push_str("  func dispatch(_ event: String) {\n");
    swift.push_str("    if let next = targetData.flow.transitions[currentState]?[event] {\n");
    swift.push_str("      currentState = next\n");
    swift.push_str("      if next.lowercased() == \"exit\" { NSApp.keyWindow?.close() }\n");
    swift.push_str("      return\n");
    swift.push_str("    }\n");
    swift.push_str("    if event == \"key(esc)\" { NSApp.keyWindow?.close() }\n");
    swift.push_str("  }\n\n");
    swift.push_str("  @ViewBuilder func renderItem(_ item: RenderItem) -> some View {\n");
    swift.push_str("    switch item.kind {\n");
    swift.push_str("    case \"button\":\n");
    swift.push_str("      Button(item.text ?? \"OK\") {\n");
    swift.push_str("        if item.action == \"modal.ok\" { showAlert = true }\n");
    swift.push_str("        if let action = item.action, !action.isEmpty {\n");
    swift.push_str("          dispatch(\"input.click(\\(action))\")\n");
    swift.push_str("        } else {\n");
    swift.push_str("          dispatch(\"input.click\")\n");
    swift.push_str("        }\n");
    swift.push_str("      }\n");
    swift.push_str("      .buttonStyle(.borderedProminent)\n");
    swift.push_str("      .controlSize(.large)\n");
    swift.push_str("      .keyboardShortcut(.defaultAction)\n");
    swift.push_str("    case \"heading\":\n");
    swift.push_str("      Text(item.text ?? \"\")\n");
    swift.push_str("        .font(.title3.weight(.semibold))\n");
    swift.push_str("        .foregroundStyle(mapColor(item.color))\n");
    swift.push_str("    case \"input\":\n");
    swift.push_str("      TextField(item.text ?? \"Input\", text: .constant(\"\"))\n");
    swift.push_str("        .textFieldStyle(.roundedBorder)\n");
    swift.push_str("        .disabled(true)\n");
    swift.push_str("    case \"checkbox\":\n");
    swift.push_str("      Toggle(item.text ?? \"Option\", isOn: .constant(false))\n");
    swift.push_str("        .disabled(true)\n");
    swift.push_str("    case \"table\":\n");
    swift.push_str("      Text(item.text ?? \"\")\n");
    swift.push_str("        .font(.system(.body, design: .monospaced))\n");
    swift.push_str("        .foregroundStyle(mapColor(item.color))\n");
    swift.push_str("    case \"modal\":\n");
    swift.push_str("      Button(item.text ?? \"Open\") {\n");
    swift.push_str("        showAlert = true\n");
    swift.push_str("      }\n");
    swift.push_str("      .buttonStyle(.bordered)\n");
    swift.push_str("    case \"panel\", \"card\":\n");
    swift.push_str("      GroupBox {\n");
    swift.push_str("        Text(item.text ?? \"\")\n");
    swift.push_str("          .foregroundStyle(mapColor(item.color))\n");
    swift.push_str("      }\n");
    swift.push_str("    default:\n");
    swift.push_str("      Text(item.text ?? \"\")\n");
    swift.push_str("        .font(mapFont(item.style))\n");
    swift.push_str("        .foregroundStyle(mapColor(item.color))\n");
    swift.push_str("    }\n");
    swift.push_str("  }\n\n");
    swift.push_str("  func mapFont(_ style: String?) -> Font {\n");
    swift.push_str("    switch style ?? \"\" {\n");
    swift.push_str("    case \"title\": return .title2.weight(.semibold)\n");
    swift.push_str("    case \"subtitle\": return .headline\n");
    swift.push_str("    case \"caption\": return .caption\n");
    swift.push_str("    default: return .body\n");
    swift.push_str("    }\n");
    swift.push_str("  }\n\n");
    swift.push_str("  func mapColor(_ color: String?) -> Color {\n");
    swift.push_str("    switch (color ?? \"\").lowercased() {\n");
    swift.push_str("    case \"yellow\": return .yellow\n");
    swift.push_str("    case \"blue\": return .blue\n");
    swift.push_str("    case \"green\": return .green\n");
    swift.push_str("    case \"red\": return .red\n");
    swift.push_str("    case \"black\": return .black\n");
    swift.push_str("    case \"white\": return .white\n");
    swift.push_str("    case \"secondary\": return .secondary\n");
    swift.push_str("    default: return .primary\n");
    swift.push_str("    }\n");
    swift.push_str("  }\n\n");
    swift.push_str("  var body: some View {\n");
    swift.push_str(&format!(
        "    VStack(alignment: {}, spacing: {}) {{\n",
        map_alignment(align),
        spacing
    ));
    swift.push_str("      ForEach(activeItems) { item in\n");
    swift.push_str("        renderItem(item)\n");
    swift.push_str("      }\n");
    swift.push_str("    }\n");
    swift.push_str(&format!("    .padding({})\n", padding));
    swift.push_str(
        "    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n",
    );
    swift.push_str(&format!(
        "    .frame(width: {}, height: {})\n",
        width, height
    ));
    swift.push_str(&format!(
        "    .background({})\n",
        map_background(background)
    ));
    swift.push_str("    .background(KeyCapture { key in dispatch(\"key(\\(key))\") }.frame(width: 0, height: 0))\n");
    swift.push_str("    .alert(\"OK\", isPresented: $showAlert) { Button(\"OK\", role: .cancel) { } }\n");
    swift.push_str("    .onExitCommand { dispatch(\"key(esc)\") }\n");
    swift.push_str("  }\n");
    swift.push_str("}\n\n");
    swift.push_str("@main struct SculptGuiApp: App {\n");
    swift.push_str("  @NSApplicationDelegateAdaptor(AppDelegate.self) var appDelegate\n");
    swift.push_str("  var body: some Scene {\n");
    swift.push_str("    WindowGroup(\"");
    swift.push_str(&escape_swift(&window_title));
    swift.push_str("\") {\n");
    swift.push_str("      ContentView()\n");
    swift.push_str("    }\n");
    swift.push_str("  }\n");
    swift.push_str("}\n");

    std::fs::write(sources_dir.join("main.swift"), swift)?;

    let status = Command::new("swift")
        .arg("build")
        .arg("-c")
        .arg("release")
        .current_dir(&app_dir)
        .status()
        .with_context(|| "Failed to run `swift build` for gui target")?;
    if !status.success() {
        bail!("gui build failed with status {:?}", status.code());
    }

    Ok(())
}

fn emit_gui_tkinter(target: &TargetIr, out_dir: &Path, data: &GuiViewData) -> Result<()> {
    let _ = data;
    emit_gui_tkinter_state_machine(out_dir, Some(target))
}

fn emit_gui_tkinter_state_machine(out_dir: &Path, target: Option<&TargetIr>) -> Result<()> {
    let gui_dir = out_dir.join("gui");
    std::fs::create_dir_all(&gui_dir)?;
    let target_json = target
        .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "{}".to_string()))
        .unwrap_or_else(|| "{}".to_string());
    let target_json_literal = serde_json::to_string(&target_json).unwrap_or_else(|_| "\"{}\"".to_string());
    let mut py = String::new();
    py.push_str("import json\n");
    py.push_str("import tkinter as tk\n");
    py.push_str("from tkinter import messagebox\n\n");
    py.push_str(&format!("TARGET = json.loads({})\n", target_json_literal));
    py.push_str("VIEWS = TARGET.get('views', {})\n");
    py.push_str("FLOW = TARGET.get('flow', {})\n");
    py.push_str("TRANSITIONS = FLOW.get('transitions', {})\n");
    py.push_str("state = FLOW.get('start', '')\n\n");
    py.push_str("root = tk.Tk()\n");
    py.push_str("window = TARGET.get('window') or {}\n");
    py.push_str("root.title(window.get('title') or 'SCULPT')\n");
    py.push_str("width = int(window.get('width') or 420)\n");
    py.push_str("height = int(window.get('height') or 260)\n");
    py.push_str("root.geometry(f'{width}x{height}')\n");
    py.push_str("root.resizable(False, False)\n\n");
    py.push_str("frame = tk.Frame(root, padx=24, pady=24)\n");
    py.push_str("frame.pack(fill='both', expand=True)\n\n");
    py.push_str("def map_color(name):\n");
    py.push_str("    c = str(name or '').lower()\n");
    py.push_str("    return {\n");
    py.push_str("        'yellow': '#ffd60a',\n");
    py.push_str("        'blue': '#0a84ff',\n");
    py.push_str("        'green': '#30d158',\n");
    py.push_str("        'red': '#ff453a',\n");
    py.push_str("        'magenta': '#ea5172',\n");
    py.push_str("        'secondary': '#9aa0aa',\n");
    py.push_str("        'white': '#ffffff',\n");
    py.push_str("        'black': '#111111',\n");
    py.push_str("    }.get(c, '#ffffff')\n\n");
    py.push_str("def map_font(style):\n");
    py.push_str("    s = str(style or '')\n");
    py.push_str("    if s == 'title': return ('Menlo', 20, 'bold')\n");
    py.push_str("    if s == 'subtitle': return ('Menlo', 13, 'bold')\n");
    py.push_str("    if s == 'caption': return ('Menlo', 11)\n");
    py.push_str("    return ('Menlo', 13)\n\n");
    py.push_str("def dispatch(event):\n");
    py.push_str("    global state\n");
    py.push_str("    next_state = (TRANSITIONS.get(state) or {}).get(event)\n");
    py.push_str("    if next_state:\n");
    py.push_str("        state = next_state\n");
    py.push_str("        if str(state).lower() == 'exit':\n");
    py.push_str("            root.destroy()\n");
    py.push_str("            return\n");
    py.push_str("        render()\n");
    py.push_str("    elif event == 'key(esc)':\n");
    py.push_str("        root.destroy()\n\n");
    py.push_str("def current_button_action():\n");
    py.push_str("    for item in VIEWS.get(state, []):\n");
    py.push_str("        if item.get('kind') == 'button':\n");
    py.push_str("            return item.get('action')\n");
    py.push_str("    return None\n\n");
    py.push_str("def on_primary(_event=None):\n");
    py.push_str("    action = current_button_action()\n");
    py.push_str("    if action == 'modal.ok':\n");
    py.push_str("        messagebox.showinfo('OK', 'OK')\n");
    py.push_str("    if action:\n");
    py.push_str("        dispatch(f\"input.click({action})\")\n");
    py.push_str("    else:\n");
    py.push_str("        dispatch('input.click')\n\n");
    py.push_str("def normalize_key(evt):\n");
    py.push_str("    k = str(evt.keysym or '').lower()\n");
    py.push_str("    if k in ('return', 'kp_enter'): return 'enter'\n");
    py.push_str("    if k == 'escape': return 'esc'\n");
    py.push_str("    if k == 'space': return 'space'\n");
    py.push_str("    if len(k) == 1: return k\n");
    py.push_str("    return k\n\n");
    py.push_str("def on_key(evt):\n");
    py.push_str("    dispatch(f\"key({normalize_key(evt)})\")\n\n");
    py.push_str("def render():\n");
    py.push_str("    for child in frame.winfo_children():\n");
    py.push_str("        child.destroy()\n");
    py.push_str("    for idx, item in enumerate(VIEWS.get(state, [])):\n");
    py.push_str("        kind = item.get('kind')\n");
    py.push_str("        text = item.get('text') or ''\n");
    py.push_str("        color = map_color(item.get('color'))\n");
    py.push_str("        if kind == 'button':\n");
    py.push_str("            tk.Button(frame, text=text or 'OK', command=on_primary).pack(anchor='w', pady=(8, 0))\n");
    py.push_str("        elif kind == 'heading':\n");
    py.push_str("            tk.Label(frame, text=text, fg=color, font=('Menlo', 16, 'bold')).pack(anchor='w', pady=(0, 8))\n");
    py.push_str("        elif kind == 'input':\n");
    py.push_str("            entry = tk.Entry(frame, width=36)\n");
    py.push_str("            entry.insert(0, text)\n");
    py.push_str("            entry.configure(state='readonly')\n");
    py.push_str("            entry.pack(anchor='w', pady=(0, 6))\n");
    py.push_str("        elif kind == 'checkbox':\n");
    py.push_str("            var = tk.BooleanVar(value=False)\n");
    py.push_str("            cb = tk.Checkbutton(frame, text=text or 'Option', variable=var)\n");
    py.push_str("            cb.configure(state='disabled')\n");
    py.push_str("            cb.pack(anchor='w', pady=(0, 6))\n");
    py.push_str("        elif kind == 'table':\n");
    py.push_str("            box = tk.Text(frame, width=44, height=6)\n");
    py.push_str("            box.insert('1.0', text)\n");
    py.push_str("            box.configure(state='disabled')\n");
    py.push_str("            box.pack(anchor='w', pady=(0, 6))\n");
    py.push_str("        elif kind in ('panel', 'card'):\n");
    py.push_str("            panel = tk.LabelFrame(frame, text='')\n");
    py.push_str("            panel.pack(anchor='w', fill='x', pady=(0, 6))\n");
    py.push_str("            tk.Label(panel, text=text, fg=color).pack(anchor='w', padx=8, pady=6)\n");
    py.push_str("        elif kind == 'modal':\n");
    py.push_str("            tk.Button(frame, text=text or 'Open', command=lambda: messagebox.showinfo('Info', text or 'OK')).pack(anchor='w', pady=(0, 6))\n");
    py.push_str("        else:\n");
    py.push_str("            tk.Label(frame, text=text, fg=color, font=map_font(item.get('style'))).pack(anchor='w', pady=(0, 8 if idx == 0 else 4))\n\n");
    py.push_str("root.bind('<KeyPress>', on_key)\n");
    py.push_str("root.bind('<Escape>', lambda _e: dispatch('key(esc)'))\n");
    py.push_str("root.bind('<Return>', on_primary)\n");
    py.push_str("root.bind('<KP_Enter>', on_primary)\n");
    py.push_str("render()\n");
    py.push_str("root.mainloop()\n");

    std::fs::write(gui_dir.join("main.py"), py)?;
    Ok(())
}

fn emit_gui_tkinter_snake(out_dir: &Path, data: &GuiViewData) -> Result<()> {
    let gui_dir = out_dir.join("gui");
    std::fs::create_dir_all(&gui_dir)?;
    let mut py = String::new();
    py.push_str("import tkinter as tk\n");
    py.push_str("import random\n\n");
    py.push_str(&format!("TITLE = \"{}\"\n", escape_py(&data.window_title)));
    py.push_str("CELL = 20\n");
    py.push_str("BOARD_W = 32\n");
    py.push_str("BOARD_H = 20\n");
    py.push_str("TICK_MS = 100\n");
    py.push_str("TARGET_SCORE = 12\n");
    py.push_str("LIVES_START = 3\n\n");
    py.push_str("root = tk.Tk()\n");
    py.push_str("root.title(TITLE)\n");
    py.push_str("root.configure(bg='#111111')\n");
    py.push_str("root.resizable(False, False)\n\n");
    py.push_str(
        "hud = tk.Label(root, text='', fg='#00ffff', bg='#111111', font=('Menlo', 12, 'bold'))\n",
    );
    py.push_str("hud.pack(pady=(8, 4))\n");
    py.push_str("canvas = tk.Canvas(root, width=BOARD_W*CELL, height=BOARD_H*CELL, bg='#0b0c10', highlightthickness=1, highlightbackground='#00ffff')\n");
    py.push_str("canvas.pack(padx=12, pady=8)\n");
    py.push_str("hint = tk.Label(root, text='Enter: start/restart  P: pause  Esc: quit  WASD/Arrows: move', fg='#00a8ff', bg='#111111', font=('Menlo', 10))\n");
    py.push_str("hint.pack(pady=(4, 10))\n\n");
    py.push_str("state = 'title'\n");
    py.push_str("direction = 'right'\n");
    py.push_str("pending = 'right'\n");
    py.push_str("snake = []\n");
    py.push_str("food = None\n");
    py.push_str("score = 0\n");
    py.push_str("lives = LIVES_START\n\n");
    py.push_str("def center_spawn():\n");
    py.push_str("    cx = BOARD_W // 2\n");
    py.push_str("    cy = BOARD_H // 2\n");
    py.push_str("    return [(cx, cy), (cx-1, cy), (cx-2, cy)]\n\n");
    py.push_str("def place_food():\n");
    py.push_str("    occupied = set(snake)\n");
    py.push_str("    free = [(x, y) for y in range(1, BOARD_H-1) for x in range(1, BOARD_W-1) if (x, y) not in occupied]\n");
    py.push_str("    return random.choice(free) if free else None\n\n");
    py.push_str("def reset_round():\n");
    py.push_str("    global snake, food, direction, pending\n");
    py.push_str("    snake = center_spawn()\n");
    py.push_str("    food = place_food()\n");
    py.push_str("    direction = 'right'\n");
    py.push_str("    pending = 'right'\n\n");
    py.push_str("def draw_cell(x, y, color):\n");
    py.push_str("    x0 = x * CELL\n");
    py.push_str("    y0 = y * CELL\n");
    py.push_str(
        "    canvas.create_rectangle(x0, y0, x0 + CELL, y0 + CELL, fill=color, width=0)\n\n",
    );
    py.push_str("def render():\n");
    py.push_str("    canvas.delete('all')\n");
    py.push_str("    for x in range(BOARD_W):\n");
    py.push_str("        draw_cell(x, 0, '#00ffff')\n");
    py.push_str("        draw_cell(x, BOARD_H-1, '#00ffff')\n");
    py.push_str("    for y in range(BOARD_H):\n");
    py.push_str("        draw_cell(0, y, '#00ffff')\n");
    py.push_str("        draw_cell(BOARD_W-1, y, '#00ffff')\n");
    py.push_str("    if food:\n");
    py.push_str("        draw_cell(food[0], food[1], '#ea5172')\n");
    py.push_str("    for i, p in enumerate(snake):\n");
    py.push_str("        draw_cell(p[0], p[1], '#ffd60a' if i == 0 else '#30d158')\n");
    py.push_str("    if state == 'title':\n");
    py.push_str("        hud.config(text='SNAKE GUI // Enter to Start', fg='#00ffff')\n");
    py.push_str("    elif state == 'pause':\n");
    py.push_str(
        "        hud.config(text=f'PAUSED  Score: {score}  Lives: {lives}', fg='#ffd60a')\n",
    );
    py.push_str("    elif state == 'gameover':\n");
    py.push_str(
        "        hud.config(text=f'GAME OVER  Score: {score}  Enter to Retry', fg='#ff453a')\n",
    );
    py.push_str("    elif state == 'victory':\n");
    py.push_str(
        "        hud.config(text=f'YOU WIN  Score: {score}  Enter to Retry', fg='#30d158')\n",
    );
    py.push_str("    else:\n");
    py.push_str("        hud.config(text=f'Score: {score}  Lives: {lives}  Length: {len(snake)}', fg='#00ffff')\n\n");
    py.push_str("def step():\n");
    py.push_str("    global state, direction, pending, snake, food, score, lives\n");
    py.push_str("    if state == 'play':\n");
    py.push_str("        direction = pending\n");
    py.push_str("        hx, hy = snake[0]\n");
    py.push_str("        nx, ny = hx, hy\n");
    py.push_str("        if direction == 'up': ny -= 1\n");
    py.push_str("        elif direction == 'down': ny += 1\n");
    py.push_str("        elif direction == 'left': nx -= 1\n");
    py.push_str("        elif direction == 'right': nx += 1\n");
    py.push_str("        if nx <= 0 or nx >= BOARD_W-1 or ny <= 0 or ny >= BOARD_H-1 or (nx, ny) in snake:\n");
    py.push_str("            lives -= 1\n");
    py.push_str("            if lives <= 0:\n");
    py.push_str("                state = 'gameover'\n");
    py.push_str("            else:\n");
    py.push_str("                reset_round()\n");
    py.push_str("        else:\n");
    py.push_str("            snake.insert(0, (nx, ny))\n");
    py.push_str("            if food and (nx, ny) == food:\n");
    py.push_str("                score += 1\n");
    py.push_str("                food = place_food()\n");
    py.push_str("                if score >= TARGET_SCORE:\n");
    py.push_str("                    state = 'victory'\n");
    py.push_str("            else:\n");
    py.push_str("                snake.pop()\n");
    py.push_str("    render()\n");
    py.push_str("    root.after(TICK_MS, step)\n\n");
    py.push_str("def start_game():\n");
    py.push_str("    global state, score, lives\n");
    py.push_str("    score = 0\n");
    py.push_str("    lives = LIVES_START\n");
    py.push_str("    reset_round()\n");
    py.push_str("    state = 'play'\n");
    py.push_str("    render()\n\n");
    py.push_str("def on_key(evt):\n");
    py.push_str("    global pending, state\n");
    py.push_str("    k = (evt.keysym or '').lower()\n");
    py.push_str("    if k == 'escape':\n");
    py.push_str("        root.destroy()\n");
    py.push_str("        return\n");
    py.push_str("    if k in ('return', 'kp_enter'):\n");
    py.push_str("        if state in ('title', 'gameover', 'victory'):\n");
    py.push_str("            start_game()\n");
    py.push_str("        elif state == 'pause':\n");
    py.push_str("            state = 'play'\n");
    py.push_str("        return\n");
    py.push_str("    if k == 'p':\n");
    py.push_str("        if state == 'play': state = 'pause'\n");
    py.push_str("        elif state == 'pause': state = 'play'\n");
    py.push_str("        return\n");
    py.push_str("    if state != 'play':\n");
    py.push_str("        return\n");
    py.push_str("    if k in ('w', 'up') and direction != 'down': pending = 'up'\n");
    py.push_str("    elif k in ('s', 'down') and direction != 'up': pending = 'down'\n");
    py.push_str("    elif k in ('a', 'left') and direction != 'right': pending = 'left'\n");
    py.push_str("    elif k in ('d', 'right') and direction != 'left': pending = 'right'\n\n");
    py.push_str("root.bind('<KeyPress>', on_key)\n");
    py.push_str("reset_round()\n");
    py.push_str("render()\n");
    py.push_str("root.after(TICK_MS, step)\n");
    py.push_str("root.mainloop()\n");
    std::fs::write(gui_dir.join("main.py"), py)?;
    Ok(())
}

fn extract_gui_view_data(target: &TargetIr) -> GuiViewData {
    GuiViewData {
        window_title: target
            .window
            .as_ref()
            .and_then(|w| w.title.clone())
            .unwrap_or_else(|| "SCULPT".to_string()),
        width: target.window.as_ref().and_then(|w| w.width).unwrap_or(420),
        height: target.window.as_ref().and_then(|w| w.height).unwrap_or(260),
    }
}

fn looks_like_snake_target(target: &TargetIr) -> bool {
    target.views.values().any(|items| {
        items.iter().any(|item| {
            item.kind == "text"
                && item
                    .text
                    .as_deref()
                    .map(|t| t.to_ascii_uppercase().contains("SNAKE"))
                    .unwrap_or(false)
        })
    })
}

fn escape_swift(input: &str) -> String {
    input.replace('\\', "\\\\").replace('\"', "\\\"")
}

fn escape_py(input: &str) -> String {
    input.replace('\\', "\\\\").replace('\"', "\\\"")
}

fn map_alignment(align: &str) -> &'static str {
    match align {
        "center" => ".center",
        "trailing" => ".trailing",
        _ => ".leading",
    }
}

fn map_background(value: &str) -> &'static str {
    match value {
        "grouped" => "Color(nsColor: .controlBackgroundColor)",
        "clear" => "Color.clear",
        _ => "Color(nsColor: .windowBackgroundColor)",
    }
}

pub fn emit_cli(target: &TargetIr, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(out_dir.join("main.js"), generate_cli_js(target))?;
    Ok(())
}

pub fn run_web(out_dir: &Path) -> Result<()> {
    let index = out_dir.join("index.html");
    if !index.exists() {
        bail!(
            "{} not found. Run `sculpt build --target web <file>` first.",
            index.display()
        );
    }
    if Command::new("open").arg(&index).status().is_ok() {
        return Ok(());
    }
    if Command::new("xdg-open").arg(&index).status().is_ok() {
        return Ok(());
    }
    if Command::new("cmd")
        .args(["/c", "start"])
        .arg(&index)
        .status()
        .is_ok()
    {
        return Ok(());
    }
    bail!(
        "Could not auto-open browser. Open {} manually.",
        index.display()
    );
}

pub fn run_gui(out_dir: &Path) -> Result<()> {
    if std::env::consts::OS == "macos" {
        let exe = out_dir
            .join("gui")
            .join(".build")
            .join("release")
            .join("SculptGui");
        if !exe.exists() {
            bail!(
                "{} not found. Run `sculpt build --target gui <file>` first.",
                exe.display()
            );
        }
        let status = Command::new(exe).status()?;
        if !status.success() {
            bail!("gui run failed with status {:?}", status.code());
        }
        return Ok(());
    }

    let script = out_dir.join("gui").join("main.py");
    if !script.exists() {
        bail!(
            "{} not found. Run `sculpt build --target gui <file>` first.",
            script.display()
        );
    }

    let status = if std::env::consts::OS == "windows" {
        Command::new("py").arg(&script).status()
    } else {
        Command::new("python3").arg(&script).status()
    }
    .with_context(|| format!("Failed to run gui target ({})", script.display()))?;
    if !status.success() {
        bail!("gui run failed with status {:?}", status.code());
    }
    Ok(())
}

pub fn run_cli(out_dir: &Path) -> Result<()> {
    let entry = out_dir.join("main.js");
    if !entry.exists() {
        bail!(
            "{} not found. Run `sculpt build --target cli <file>` first.",
            entry.display()
        );
    }
    let status = Command::new("node")
        .arg(&entry)
        .status()
        .with_context(|| format!("Failed to run cli target (node {})", entry.display()))?;
    if !status.success() {
        bail!("cli run failed with status {:?}", status.code());
    }
    Ok(())
}

pub fn list_targets() -> Result<Vec<String>> {
    let mut targets = vec!["cli".to_string(), "web".to_string(), "gui".to_string()];

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if let Some(rest) = name.strip_prefix("sculpt-target-") {
                            if !rest.is_empty() {
                                targets.push(rest.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    targets.sort();
    targets.dedup();
    Ok(targets)
}

pub fn describe_target(name: &str) -> Result<Value> {
    match resolve_target(name) {
        TargetKind::Web => builtin_spec(
            "web-ir",
            include_str!("../../ir-schemas/web-ir.json"),
            Some(vec![
                "runtime.web",
                "render.text",
                "input.key",
                "flow.state_machine",
                "web.profile",
                "web.adapter",
            ]),
            Some(json!({
              "layout": { "type": "enum", "values": ["default"] },
              "web_profile": { "type": "enum", "values": ["standard", "next-app", "laravel-mvc"] }
            })),
            Some(json!({
              "runtime": ["browser"],
              "adapters": [
                { "id": "builtin.web.standard@1", "class": "frontend", "description": "Built-in browser runtime emitter" },
                { "id": "provider.web.next@1", "class": "frontend", "description": "External Next.js adapter provider" },
                { "id": "provider.web.laravel@1", "class": "backend", "description": "External Laravel MVC adapter provider" }
              ],
              "standard_app_ir": "web-app-ir"
            })),
            None,
            Some(json!([
              {
                "id": "builtin.web.ui@1",
                "namespace": "ui",
                "description": "Web rendering primitives",
                "exports": [
                  "text",
                  "heading",
                  "button",
                  "badge",
                  "list",
                  "table",
                  "input",
                  "textarea",
                  "select",
                  "checkbox",
                  "radio",
                  "panel",
                  "card",
                  "tabs",
                  "modal",
                  "toast",
                  "banner",
                  "progress",
                  "metric",
                  "chart",
                  "link",
                  "image"
                ]
              },
              {
                "id": "builtin.web.input@1",
                "namespace": "input",
                "description": "Web input events",
                "exports": [
                  "key",
                  "click",
                  "submit",
                  "change",
                  "focus",
                  "blur",
                  "navigate",
                  "back",
                  "refresh"
                ]
              },
              {
                "id": "builtin.web.data@1",
                "namespace": "data",
                "description": "Data and query primitives",
                "exports": [
                  "csvRead",
                  "rowCount",
                  "csvHasColumns",
                  "csvMissingColumns",
                  "schemaErrorMessage",
                  "reconcileInvoices",
                  "metric",
                  "buildExceptions",
                  "buildReportJson",
                  "processingMs",
                  "writeJson",
                  "sortBy",
                  "writeCsv",
                  "summaryLine",
                  "query",
                  "mutate",
                  "filter",
                  "sort",
                  "paginate",
                  "group",
                  "aggregate",
                  "join"
                ]
              },
              {
                "id": "builtin.web.net@1",
                "namespace": "net",
                "description": "HTTP/API integration primitives",
                "exports": ["get", "post", "put", "patch", "delete", "upload", "download"]
              },
              {
                "id": "builtin.web.guide@1",
                "namespace": "guide",
                "description": "Web ND constraint vocabulary",
                "exports": [
                  "clearPriorityContrast",
                  "keyboardFirstNavigation",
                  "readableOnLaptopScreens",
                  "mobileFirstLayout",
                  "desktopDensityBalanced",
                  "responsiveBreakpoints",
                  "noOverlap",
                  "clearSeverityHierarchy",
                  "clearInformationHierarchy",
                  "keyboardNavigable",
                  "mobileFallbackExists",
                  "accessibleColorContrast",
                  "formValidationClarity"
                ]
              }
            ])),
        ),
        TargetKind::Cli => builtin_spec(
            "cli-ir",
            include_str!("../../ir-schemas/cli-ir.json"),
            Some(vec![
                "runtime.cli",
                "runtime.rules",
                "runtime.when.logic",
                "render.text",
                "input.key",
                "flow.state_machine",
            ]),
            Some(json!({
              "layout": { "type": "enum", "values": ["default"] }
            })),
            Some(json!({ "runtime": ["desktop", "server"] })),
            Some(json!({
              "runtimeRules": {
                "type": "array",
                "items": {
                  "type": "object",
                  "required": ["name", "assign", "emit"],
                  "properties": {
                    "name": { "type": "string" },
                    "scopeFlow": { "type": ["string", "null"] },
                    "scopeState": { "type": ["string", "null"] },
                    "on": { "type": ["string", "null"] },
                    "when": { "type": ["object", "null"] },
                    "assign": { "type": "array" },
                    "emit": { "type": "array" }
                  },
                  "additionalProperties": true
                }
              }
            })),
            Some(json!([
              {
                "id": "builtin.cli.ui@1",
                "namespace": "ui",
                "description": "CLI rendering primitives",
                "exports": [
                  "text",
                  "line",
                  "clear",
                  "panel",
                  "list",
                  "table",
                  "progress",
                  "status",
                  "banner",
                  "separator",
                  "metric",
                  "chart"
                ]
              },
              {
                "id": "builtin.cli.input@1",
                "namespace": "input",
                "description": "CLI input events",
                "exports": [
                  "key",
                  "tick",
                  "submit",
                  "confirm",
                  "select",
                  "cancel",
                  "resize"
                ]
              },
              {
                "id": "builtin.cli.data@1",
                "namespace": "data",
                "description": "Deterministic data and batch processing primitives",
                "exports": [
                  "csvRead",
                  "rowCount",
                  "csvHasColumns",
                  "csvMissingColumns",
                  "schemaErrorMessage",
                  "reconcileInvoices",
                  "metric",
                  "buildExceptions",
                  "buildReportJson",
                  "processingMs",
                  "writeJson",
                  "sortBy",
                  "writeCsv",
                  "summaryLine"
                ]
              },
              {
                "id": "builtin.cli.guide@1",
                "namespace": "guide",
                "description": "CLI ND constraint vocabulary",
                "exports": [
                  "playable",
                  "menuClarity",
                  "readableHud",
                  "readableTableLayout",
                  "responsiveControls",
                  "visuallyDistinctHeadAndFood",
                  "smoothDifficultyCurve",
                  "clearWinOrLossFeedback",
                  "loopedGameplay",
                  "compactTerminalLayout",
                  "exact",
                  "exactPalette",
                  "highContrast",
                  "lowFlickerOutput",
                  "clearOperationalSummary",
                  "professionalTone",
                  "conciseLanguage",
                  "noLegalRiskTerms",
                  "hasClearTitle",
                  "hasActionableSteps",
                  "usesOperationalLanguage",
                  "supportsQuickKeyNavigation",
                  "fullyInsideBounds",
                  "mirroredDifficultyCurve",
                  "guaranteedLaunchLane",
                  "noUnreachableBricks",
                  "firstLevelIsForgiving",
                  "visualPalette",
                  "followsClassicBreakoutRules",
                  "hasPaddleBallBrickLoop",
                  "supportsControls",
                  "launchesBallOnSpace",
                  "bouncesOnWallsAndPaddle",
                  "removesBricksOnImpact",
                  "tracksScoreAndLives",
                  "emitsWinWhenBricksCleared",
                  "emitsDoneWhenLivesDepleted",
                  "preservesArcadePacing",
                  "includesReadableHud",
                  "usesHighContrastTerminalColors",
                  "noSoftLocks",
                  "deterministicCoreWithOptionalStyleVariance",
                  "startsForgiving",
                  "increasesAfterEachStage",
                  "keepsRunDurationReasonable"
                ]
              }
            ])),
        ),
        TargetKind::Gui => builtin_spec(
            "gui-ir",
            include_str!("../../ir-schemas/gui-ir.json"),
            Some(vec![
                "runtime.gui",
                "render.text",
                "input.key",
                "flow.state_machine",
                "layout.explicit",
                "ui.modal.ok",
            ]),
            Some(json!({
              "layout": { "type": "enum", "values": ["default", "explicit"] }
            })),
            Some(json!({
              "runtime": ["desktop"],
              "backends": {
                "macos": "swiftui-swiftpm",
                "windows": "python-tkinter",
                "linux": "python-tkinter"
              }
            })),
            None,
            Some(json!([
              {
                "id": "builtin.gui.ui@1",
                "namespace": "ui",
                "description": "GUI rendering primitives",
                "exports": [
                  "text",
                  "heading",
                  "button",
                  "input",
                  "textarea",
                  "select",
                  "checkbox",
                  "radio",
                  "image",
                  "icon",
                  "list",
                  "table",
                  "panel",
                  "card",
                  "tabs",
                  "spacer",
                  "progress",
                  "status"
                ]
              },
              {
                "id": "builtin.gui.input@1",
                "namespace": "input",
                "description": "GUI input events",
                "exports": ["key", "click", "submit", "change", "focus", "blur", "closeWindow"]
              },
              {
                "id": "builtin.gui.window@1",
                "namespace": "window",
                "description": "Window and modal controls",
                "exports": ["open", "close", "resize", "modalOk", "modalConfirm", "notify"]
              },
              {
                "id": "builtin.gui.data@1",
                "namespace": "data",
                "description": "Deterministic data and batch processing primitives",
                "exports": [
                  "csvRead",
                  "rowCount",
                  "csvHasColumns",
                  "csvMissingColumns",
                  "schemaErrorMessage",
                  "reconcileInvoices",
                  "metric",
                  "buildExceptions",
                  "buildReportJson",
                  "processingMs",
                  "writeJson",
                  "sortBy",
                  "writeCsv",
                  "summaryLine"
                ]
              },
              {
                "id": "builtin.gui.guide@1",
                "namespace": "guide",
                "description": "GUI ND constraint vocabulary",
                "exports": [
                  "highContrast",
                  "professionalTone",
                  "clearOperationalSummary",
                  "desktopNativeLook",
                  "focusOrderStable",
                  "dialogCopyClarity"
                ]
              }
            ])),
        ),
        TargetKind::External(t) => external_describe(&t),
    }
}

fn builtin_spec(
    standard_ir: &str,
    schema: &str,
    capabilities: Option<Vec<&str>>,
    target_meta: Option<Value>,
    support: Option<Value>,
    extensions_schema: Option<Value>,
    packages: Option<Value>,
) -> Result<Value> {
    let schema_json: Value = serde_json::from_str(schema)?;
    let mut meta = json!({
      "target": { "type": "string" },
      "profile": { "type": "enum", "values": ["default", "portable"] },
      "nd_policy": { "type": "enum", "values": ["strict"] },
      "strict_scopes": { "type": "bool" },
      "nd_budget": { "type": "int", "min": 0, "max": 100 },
      "confidence": { "type": "float", "min": 0.0, "max": 1.0 },
      "requires": { "type": "capability_list" },
      "max_iterations": { "type": "int", "min": 1, "max": 10000 },
      "fallback": { "type": "enum", "values": ["fail", "stub", "replay"] }
    });
    if let Some(target_meta) = target_meta {
        if let (Some(meta_obj), Some(target_obj)) = (meta.as_object_mut(), target_meta.as_object())
        {
            for (k, v) in target_obj {
                meta_obj.insert(k.clone(), v.clone());
            }
        }
    }
    let mut value = json!({
      "standard_ir": standard_ir,
      "schema": schema_json,
      "extensions": {},
      "contract": {
        "version": 1,
        "capabilities": capabilities
          .unwrap_or_default()
          .into_iter()
          .map(|s| Value::String(s.to_string()))
          .collect::<Vec<_>>(),
        "meta": meta,
        "extensions_schema": extensions_schema.unwrap_or_else(|| json!({})),
        "packages": packages.unwrap_or_else(|| json!([]))
      }
    });
    if let Some(support) = support {
        value["support"] = support;
    }
    Ok(value)
}

fn external_describe(target: &str) -> Result<Value> {
    let exe = format!("sculpt-target-{}", target);
    let output = Command::new(&exe)
        .arg("describe")
        .output()
        .with_context(|| format!("Failed to launch target provider: {}", exe))?;
    if !output.status.success() {
        bail!("Target provider {} describe failed", exe);
    }
    let value: Value = serde_json::from_slice(&output.stdout)?;
    Ok(value)
}
