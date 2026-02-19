# SCULPT

(C) 2026 byte5 GmbH

**SCULPT — an intent-oriented, AI-native convergent programming language.**

SCULPT lets developers shape software through structured non-deterministic code instead of prose prompts.
The compiler narrows solution space through constraints, compiles via LLM to target IR, and target providers produce deterministic build artifacts.

**AI‑first, but human‑centered:** built *by* AI, built *with* AI, built *for* humans.

## What This Repository Contains
- Rust CLI compiler (`sculpt`)
- Interactive TUI mode
- LLM provider integration (`openai`, `anthropic`, `gemini`, `stub`)
- Built-in target providers (`cli`, `gui`, `web`)
- Example SCULPT programs

## Thesis
LLM cost and speed are not static. We expect a trajectory similar to bandwidth on the early internet:
as models become faster and cheaper, what is possible expands dramatically. SCULPT therefore optimizes
for today’s constraints without sacrificing tomorrow’s capability.

## Status
- CLI compiler with LLM-backed build pipeline.
- Built-in targets: `cli`, `gui`, `web`.
  - `gui` builds:
    - macOS: native SwiftUI app (SwiftPM)
    - Windows/Linux: Python Tkinter desktop app (MVP parity path)

## Start Here
- `SCULPT_Quick_Start.md`
- `SCULPT_Handbook.md`

## Documentation Map
To keep documentation consistent, each document has one clear purpose:

- `SCULPT_Quick_Start.md`: installation, first build, first run.
- `SCULPT_Handbook.md`: practical guide to compiler workflow and language usage.
- `SCULPT_Syntax_Manifest.md`: syntax only (grammar-level rules).
- `SCULPT_Semantics.md`: runtime/validation semantics and diagnostic model.
- `SCULPT_Namespaces_And_Scopes.md`: namespace model, symbol resolution, and scope policy.
- `SCULPT_Target_Model.md`: intent/runtime/provider architecture for future-proof targets.
- `SCULPT_Professional_Grade_Blueprint.md`: roadmap for large multi-team systems.
- `SCULPT_Backlog.md`: prioritized implementation backlog.

## License
MIT
