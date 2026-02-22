# SCULPT

(C) 2026 byte5 GmbH

**SCULPT — an intent-oriented, AI-native convergent programming language.**

SCULPT lets developers shape software through structured non-deterministic code instead of prose prompts.
The compiler narrows solution space through constraints, compiles via LLM to target IR, and target providers build deterministic artifacts.

**AI‑first, but human‑centered:** built *by* AI, built *with* AI, built *for* humans.

## Elevator Pitch
SCULPT is for developers who want the speed of AI coding without giving up control.
You write structured code (flows, states, rules, constraints), not long prompt prose.
SCULPT then converges that intent through an LLM into target IR and produces reproducible builds for `cli`, `gui`, and `web`.

## What This Repository Contains
- Rust CLI compiler (`sculpt`)
- Interactive TUI mode
- LLM provider integration (`openai`, `anthropic`, `gemini`, `stub`)
- Built-in target providers (`cli`, `gui`, `web`)
- Example SCULPT programs

## Why SCULPT
In mainstream software delivery, fully hand-written code is becoming the exception.
AI-assisted development is growing, but prose-prompt workflows are hard to control and hard to scale in teams.
SCULPT targets that gap with code-first convergence and deterministic build paths.

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
- [SCULPT Quick Start](docs/SCULPT_Quick_Start.md)
- [SCULPT For Dummies](docs/SCULPT_For_Dummies.md)
- [SCULPT Handbook](docs/SCULPT_Handbook.md)
- [SCULPT Roadmap](docs/SCULPT_Roadmap.md)
- [SCULPT Case Studies Overview](poc/SCULPT_Case_Studies_Overview.md)

## Documentation Map
Documentation has been moved under `/docs` for easier navigation:
- [Docs Index](docs/README.md)
- [Versioning Policy](docs/SCULPT_Versioning.md)

## License
MIT
