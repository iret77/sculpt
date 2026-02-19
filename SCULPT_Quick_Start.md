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

This writes sample `.sculpt` programs into `/examples`.

## 4) Configure an LLM Provider
Use one of:
- `GEMINI_API_KEY`
- `OPENAI_API_KEY`
- `ANTHROPIC_API_KEY`

Optional config file:
`sculpt.config.json`

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
sculpt build examples/hello_world.sculpt --target cli
```

Run:

```bash
sculpt run examples/hello_world.sculpt --target cli
```

If `@meta target=...` is set in the script, you can omit `--target`.

## 6) TUI Mode
Start interactive mode:

```bash
sculpt
```

Core keys:
- `Enter`: run or build+run for selected file
- `B`: build+run
- `R`: run only (if executable artifact exists)
- `F`: freeze
- `P`: replay
- `C`: clean selected script artifacts
- `Esc`: quit

## 7) Build Artifacts
Outputs are isolated per script:

- `dist/<script_name>/ir.json`
- `dist/<script_name>/target.ir.json`
- `dist/<script_name>/nondet.report`
- `dist/<script_name>/build.meta.json`

## 8) Debug Output

```bash
sculpt build <file.sculpt> --target cli --debug
sculpt build <file.sculpt> --target cli --debug=all
```

For full command reference and workflow details, see:
- `SCULPT_Handbook.md`
