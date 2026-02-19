# SCULPT

(C) 2026 byte5 GmbH

**SCULPT — a convergent programming language.**

SCULPT is an AI-first compiler and language for convergent programming.
You write structured non-deterministic code, the compiler narrows it via constraints, an LLM generates target IR, and target providers build deterministic outputs.

**AI‑first, but human‑centered:** build *by* AI, build *with* AI, build *for* humans.

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
  - `gui` currently generates a macOS SwiftUI app via SwiftPM.

## Start Here
- `/Users/cwendler/Dev/App/sculpt/SCULPT_Quick_Start.md`
- `/Users/cwendler/Dev/App/sculpt/SCULPT_Handbook.md`

## Documentation Map
To keep documentation consistent, each document has one clear purpose:

- `/Users/cwendler/Dev/App/sculpt/SCULPT_Quick_Start.md`: installation, first build, first run.
- `/Users/cwendler/Dev/App/sculpt/SCULPT_Handbook.md`: practical guide to compiler workflow and language usage.
- `/Users/cwendler/Dev/App/sculpt/SCULPT_Syntax_Manifest.md`: syntax only (grammar-level rules).
- `/Users/cwendler/Dev/App/sculpt/SCULPT_Semantics.md`: runtime/validation semantics and diagnostic model.
- `/Users/cwendler/Dev/App/sculpt/SCULPT_Namespaces_And_Scopes.md`: namespace model, symbol resolution, and scope policy.
- `/Users/cwendler/Dev/App/sculpt/SCULPT_Professional_Grade_Blueprint.md`: roadmap for large multi-team systems.
- `/Users/cwendler/Dev/App/sculpt/SCULPT_Backlog.md`: prioritized implementation backlog.

## License
MIT
