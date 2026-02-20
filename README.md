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

## Extended Thesis
In mainstream software delivery, fully hand-written code is becoming the exception.
AI-assisted development is the default direction, but prose-prompt workflows remain hard to control and hard to scale in teams.

SCULPT targets that gap:
- code-native instead of prose-native,
- intent-oriented instead of prompt-chaotic,
- convergent instead of drift-prone,
- reproducible instead of ephemeral.

## Status
- CLI compiler with LLM-backed build pipeline.
- Built-in targets: `cli`, `gui`, `web`.
  - `gui` builds:
    - macOS: native SwiftUI app (SwiftPM)
    - Windows/Linux: Python Tkinter desktop app (MVP parity path)

## Start Here
- [SCULPT Quick Start](docs/SCULPT_Quick_Start.md)
- [SCULPT For Dummies](docs/SCULPT_For_Dummies.md)
- [SCULPT Handbook](docs/SCULPT_Handbook.md)
- [SCULPT Roadmap](docs/SCULPT_Roadmap.md)
- [SCULPT Case Studies Overview](poc/SCULPT_Case_Studies_Overview.md)

## Documentation Map
Documentation has been moved under `/docs` for easier navigation:
- [Docs Index](docs/README.md)

## License
MIT
