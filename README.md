# SCULPT

(C) 2026 byte5 GmbH

**SCULPT — a convergent programming language.**

SCULPT is a research compiler for a converging, AI-first programming language.
You write structured, non-deterministic code; the compiler turns it into a compact IR
and asks an LLM to produce target IR, which is then built deterministically by a target provider.

**AI‑first, but human‑centered:** build *by* AI, build *with* AI, build *for* humans.

The LLM sees **only** a compact, positional‑array IR. We then normalize into standard IR
for deterministic build and validation.

## Thesis
LLM cost and speed are not static. We expect a trajectory similar to bandwidth on the early internet:
as models become faster and cheaper, what is possible expands dramatically. SCULPT therefore optimizes
for today’s constraints without sacrificing tomorrow’s capability.

This repo contains the compiler in Rust and a small set of example programs.

## Why SCULPT
- **Converging programming**: fewer lines mean a larger solution space; more lines constrain it.
- **AI-first**: the compiler, not the developer, handles prompting and target translation.
- **Provider-based**: LLM providers + target providers are pluggable.

## Status
- Research compiler (CLI) with LLM-backed build pipeline.
- Built-in targets: `cli`, `gui`, `web`.
  - `gui` currently generates a macOS SwiftUI app via SwiftPM.

## Install
```bash
cargo install --path .
```

## Quick Start
Write or use an example file, then build:
```bash
sculpt examples
sculpt build examples/hello_world.sculpt --target cli --provider openai
```

Run the output:
```bash
sculpt run examples/hello_world.sculpt --target cli
```

## Commands
- `sculpt examples` — write example programs into `examples/`
- `sculpt build <file.sculpt> --target <cli|gui|web> --provider <openai|anthropic|gemini|stub>`
- `sculpt freeze <file.sculpt> --target <...> --provider <...>`
- `sculpt replay <file.sculpt> --target <...>`
- `sculpt run <file.sculpt> --target <...>`
- `sculpt target list`
- `sculpt target describe --target <name>`
- `sculpt auth check --provider openai [--verify]`
- `sculpt build <file.sculpt> --target <...> --debug[=compact|raw|all|json]`

## Authentication
Set a provider API key as an environment variable:
```bash
export OPENAI_API_KEY=... 
export ANTHROPIC_API_KEY=...
export GEMINI_API_KEY=...
```

Optional config file (overridden by env var). `provider` acts as the default, so you can omit `--provider`.
If `--model` is omitted, the per‑provider model below is used (otherwise the built‑in fallback applies).
```json
{
  "provider": "gemini",
  "openai":   { "api_key": "...", "model": "gpt-4.1-mini" },
  "anthropic":{ "api_key": "...", "model": "claude-sonnet-4-20250514" },
  "gemini":   { "api_key": "...", "model": "gemini-2.5-pro" }
}
```

## Examples
- `examples/hello_world.sculpt` — deterministic hello world
- `examples/snake_high_nd.sculpt` — high ND, large solution space
- `examples/snake_low_nd.sculpt` — low ND, tight solution space
- `examples/invoice_review.sculpt` — business/web flavored example
- `examples/native_window.sculpt` — macOS GUI window + button demo

## Syntax
See `SCULPT_Syntax_Manifest.md` for the current syntax.

## License
MIT
