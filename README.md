# SCULPT

(C) 2026 byte5 GmbH

## Elevator Pitch
Prompt-first AI coding is fast at first and expensive later: unclear intent, brittle outputs, weak reviewability, and poor reproducibility.  
SCULPT fixes that by making AI generation code-native and constraint-driven.  
Instead of wrestling with long prompt chains, teams get a compile pipeline that is inspectable, repeatable, and built for real software delivery.

**SCULPT — an intent-oriented, AI-native convergent programming language.**

SCULPT is a code-first layer above prompting: you shape solution space, the compiler converges it through LLMs, and target providers build deterministic outputs for `cli`, `gui`, and `web`.
**AI‑first, but human‑centered:** built *by* AI, built *with* AI, built *for* humans.

## What This Repository Contains
- Rust CLI compiler (`sculpt`)
- Interactive TUI mode
- LLM provider integration (`openai`, `anthropic`, `gemini`, `stub`)
- Built-in target providers (`cli`, `gui`, `web`)
- Example SCULPT programs

## Why SCULPT
In mainstream delivery, fully hand-written code is becoming the exception.
SCULPT gives teams a better default than prompt-only workflows: faster iteration with stronger control.

- code-native instead of prose-native,
- intent-oriented instead of prompt-chaotic,
- convergent instead of drift-prone,
- reproducible instead of ephemeral.

## Status
- CLI compiler with LLM-backed build pipeline.
- Built-in targets: `cli`, `gui`, `web`.
  - `gui` builds:
    - macOS: native SwiftUI app (SwiftPM)
    - Windows/Linux: Python Tkinter desktop app (cross-platform parity path)

## Start Here
- I am curious and need a simple walkthrough:
  - [SCULPT For Dummies](docs/SCULPT_For_Dummies.md)
- I want to run SCULPT now:
  - [SCULPT Quick Start](docs/SCULPT_Quick_Start.md)
- I want to see concrete SCULPT code:
  - [Examples](examples/README.md)
- I want implementation details:
  - [SCULPT Handbook](docs/SCULPT_Handbook.md)
  - [Docs Index](docs/README.md)
- I want evidence and strategic direction:
  - [SCULPT Case Studies Overview](poc/SCULPT_Case_Studies_Overview.md)
  - [SCULPT Roadmap](docs/SCULPT_Roadmap.md)

## Documentation Map
Documentation has been moved under `/docs` for easier navigation:
- [Docs Index](docs/README.md)
- [Versioning Policy](docs/SCULPT_Versioning.md)

## License
MIT
