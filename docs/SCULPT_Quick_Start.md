# SCULPT Quick Start

(C) 2026 byte5 GmbH

This guide is for developers who want to run SCULPT locally in a few minutes.

## 1) Prerequisites
- Rust + Cargo installed
- Node.js installed (for `cli`/`web` target run helpers)
- `gui` target requirements:
  - macOS: Xcode command line tools (Swift build)
  - Windows/Linux: Python (Tkinter)

## 2) Install
From the repository root:

```bash
cargo install --path .
```

Verify:

```bash
sculpt --version
sculpt help
```

## 3) Create Example Files

```bash
sculpt examples
```

This writes sample `.sculpt` programs into `/examples` (grouped by domain).

## 4) Configure an LLM Provider
Use one of:
- `GEMINI_API_KEY`
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`

Optional config file:
`sculpt.config.json`

You can start from:
`sculpt.config.example.json`

Example:

```json
{
  "provider": "gemini",
  "gemini": { "api_key": "YOUR_KEY", "model": "gemini-2.5-pro" }
}
```

## 5) Build and Run

Build:

```bash
sculpt build examples/getting-started/hello_world.sculpt --target cli
```

Optional ND token policy override:

```bash
sculpt build examples/getting-started/hello_world.sculpt --target cli --nd-policy strict
```

Run:

```bash
sculpt run examples/getting-started/hello_world.sculpt --target cli
```

If `@meta target=...` is set in the script, you can omit `--target`.

## 6) Create a Project File (Multi-File)

If your scripts use `import(...)`, build via `.sculpt.json` project files.

Create one automatically:

```bash
sculpt project create billing -p examples/business -f "*.sculpt"
```

Build/run project:

```bash
sculpt build examples/business/billing.sculpt.json --provider stub
sculpt run examples/business/billing.sculpt.json
```

## 7) TUI Mode
Start interactive mode:

```bash
sculpt
```

Core keys:
- `Enter`: run or build+run for selected file/project
- `B`: build+run
- `R`: run only (if executable artifact exists)
- `F`: freeze
- `P`: replay
- `C`: clean selected script/project artifacts
- `Esc`: quit

## 8) Build Artifacts
Outputs are isolated per script:

- `dist/<script_name>/ir.json`
- `dist/<script_name>/target.ir.json`
- `dist/<script_name>/nondet.report`
- `dist/<script_name>/build.meta.json`

For project files (`*.sculpt.json`) outputs go to:
- `dist/<project_name>/...`

## 9) Debug Output

```bash
sculpt build <input.sculpt|project.sculpt.json> --target cli --debug
sculpt build <input.sculpt|project.sculpt.json> --target cli --debug=all
```

For full command reference and workflow details, see:
- [SCULPT Handbook](SCULPT_Handbook.md)
