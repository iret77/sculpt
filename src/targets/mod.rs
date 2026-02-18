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
    bail!("Target provider {} failed with status {:?}", exe, status.code());
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
    <title>Sculpt Web Target</title>
    <style>
      body { font-family: sans-serif; padding: 24px; }
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

  let view_name = target.flow.start.clone();
  let items = target.views.get(&view_name).cloned().unwrap_or_default();
  let mut text_views: Vec<(String, String, Option<String>)> = Vec::new();
  let mut button_label = None;
  let mut button_action = None;

  for item in items {
    match item.kind.as_str() {
      "text" => {
        if let Some(text) = item.text {
          let color = item.color.unwrap_or_else(|| "primary".to_string());
          text_views.push((text, color, item.style));
        }
      }
      "button" => {
        button_label = item.text.or(Some("OK".to_string()));
        if let Some(action) = item.action {
          button_action = Some(action);
        }
      }
      _ => {}
    }
  }

  if button_label.is_none() {
    button_label = Some("OK".to_string());
  }

  let window_title = target
    .window
    .as_ref()
    .and_then(|w| w.title.clone())
    .unwrap_or_else(|| "SCULPT".to_string());
  let width = target.window.as_ref().and_then(|w| w.width).unwrap_or(420);
  let height = target.window.as_ref().and_then(|w| w.height).unwrap_or(260);
  let layout = target
    .layout
    .as_ref()
    .and_then(|map| map.get(&view_name));
  let padding = layout.and_then(|l| l.padding).unwrap_or(24);
  let spacing = layout.and_then(|l| l.spacing).unwrap_or(12);
  let align = layout.and_then(|l| l.align.as_deref()).unwrap_or("leading");
  let background = layout.and_then(|l| l.background.as_deref()).unwrap_or("window");

  let mut swift = String::new();
  swift.push_str("import SwiftUI\nimport AppKit\n\n");
  swift.push_str("final class AppDelegate: NSObject, NSApplicationDelegate {\n");
  swift.push_str("  func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool { true }\n");
  swift.push_str("}\n\n");
  swift.push_str("struct ContentView: View {\n");
  let show_alert = matches!(button_action.as_deref(), Some("modal.ok"));
  if show_alert {
    swift.push_str("  @State private var showAlert = false\n\n");
  }
  swift.push_str("  var body: some View {\n");
  swift.push_str(&format!("    VStack(alignment: {}, spacing: {}) {{\n", map_alignment(align), spacing));
  for (idx, (text, color, style)) in text_views.iter().enumerate() {
    let (font, fallback_color) = map_text_style(style.as_deref(), idx);
    let mapped = map_color_or(color, fallback_color);
    swift.push_str(&format!(
      "      Text(\"{}\"){font}.foregroundStyle({})\n",
      escape_swift(text),
      mapped
    ));
  }
  let button_action_code = if show_alert { "{ showAlert = true }" } else { "{}" };
  swift.push_str(&format!(
    "      Button(\"{}\") {}\n",
    escape_swift(button_label.as_deref().unwrap_or("OK")),
    button_action_code
  ));
  swift.push_str("        .buttonStyle(.borderedProminent)\n");
  swift.push_str("        .controlSize(.large)\n");
  swift.push_str("        .keyboardShortcut(.defaultAction)\n");
  swift.push_str("    }\n");
  swift.push_str(&format!("    .padding({})\n", padding));
  swift.push_str("    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)\n");
  swift.push_str(&format!("    .frame(width: {}, height: {})\n", width, height));
  swift.push_str(&format!("    .background({})\n", map_background(background)));
  if show_alert {
    swift.push_str("    .alert(\"OK\", isPresented: $showAlert) { Button(\"OK\", role: .cancel) { } }\n");
  }
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

fn escape_swift(input: &str) -> String {
  input.replace('\\', "\\\\").replace('\"', "\\\"")
}

fn map_color_or(color: &str, fallback: &str) -> String {
  match color.to_lowercase().as_str() {
    "yellow" => "Color.yellow".to_string(),
    "blue" => "Color.blue".to_string(),
    "green" => "Color.green".to_string(),
    "red" => "Color.red".to_string(),
    "black" => "Color.black".to_string(),
    "white" => "Color.white".to_string(),
    "primary" => "Color.primary".to_string(),
    "secondary" => "Color.secondary".to_string(),
    _ => format!("Color.{}", fallback),
  }
}

fn map_text_style(style: Option<&str>, index: usize) -> (&'static str, &'static str) {
  match style.unwrap_or("") {
    "title" => (".font(.title2.weight(.semibold))", "primary"),
    "subtitle" => (".font(.headline)", "secondary"),
    "caption" => (".font(.caption)", "secondary"),
    "body" => (".font(.body)", "secondary"),
    _ => {
      if index == 0 {
        (".font(.title2.weight(.semibold))", "primary")
      } else {
        (".font(.body)", "secondary")
      }
    }
  }
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
    bail!("dist/index.html not found. Run `sculpt build --target web <file>` first.");
  }
  if Command::new("open").arg(&index).status().is_ok() {
    return Ok(());
  }
  if Command::new("xdg-open").arg(&index).status().is_ok() {
    return Ok(());
  }
  if Command::new("cmd").args(["/c", "start"]).arg(&index).status().is_ok() {
    return Ok(());
  }
  bail!("Could not auto-open browser. Open dist/index.html manually.");
}

pub fn run_gui(out_dir: &Path) -> Result<()> {
  let exe = out_dir.join("gui").join(".build").join("release").join("SculptGui");
  if !exe.exists() {
    bail!("dist/gui/.build/release/SculptGui not found. Run `sculpt build --target gui <file>` first.");
  }
  let status = Command::new(exe).status()?;
  if !status.success() {
    bail!("gui run failed with status {:?}", status.code());
  }
  Ok(())
}

pub fn run_cli(out_dir: &Path) -> Result<()> {
  let entry = out_dir.join("main.js");
  if !entry.exists() {
    bail!("dist/main.js not found. Run `sculpt build --target cli <file>` first.");
  }
  let status = Command::new("node")
    .arg(entry)
    .status()
    .with_context(|| "Failed to run cli target (node dist/main.js)")?;
  if !status.success() {
    bail!("cli run failed with status {:?}", status.code());
  }
  Ok(())
}

pub fn list_targets() -> Result<Vec<String>> {
  let mut targets = vec![
    "cli".to_string(),
    "web".to_string(),
    "gui".to_string(),
  ];

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
    TargetKind::Web => builtin_spec("web-ir", include_str!("../../ir-schemas/web-ir.json")),
    TargetKind::Cli => builtin_spec("cli-ir", include_str!("../../ir-schemas/cli-ir.json")),
    TargetKind::Gui => builtin_spec("gui-ir", include_str!("../../ir-schemas/gui-ir.json")),
    TargetKind::External(t) => external_describe(&t),
  }
}

fn builtin_spec(standard_ir: &str, schema: &str) -> Result<Value> {
  let schema_json: Value = serde_json::from_str(schema)?;
  Ok(json!({
    "standard_ir": standard_ir,
    "schema": schema_json,
    "extensions": {}
  }))
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
