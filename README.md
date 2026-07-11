# SCULPT

(C) 2026 byte5 GmbH

## Elevator Pitch
**Prompts tell AI what you want. SCULPT lets you program what AI is allowed to build.**

SCULPT is the convergent programming language for the AI era. Developers encode
behavior, constraints, freedoms, and proof as versioned source code; compiler
agents generate and refine the implementation until every required obligation
is satisfied.

It gives professional developers the speed of AI generation without
surrendering software architecture to prose, conversation history, and luck.

[Read the Convergent Programming concept](docs/SCULPT_Convergent_Programming_Concept.md).

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
  - [SCULPT Convergent Programming Concept](docs/SCULPT_Convergent_Programming_Concept.md)
  - [SCULPT Case Studies Overview](poc/SCULPT_Case_Studies_Overview.md)
  - [SCULPT Roadmap](docs/SCULPT_Roadmap.md)

## Documentation Map
Documentation has been moved under `/docs` for easier navigation:
- [Docs Index](docs/README.md)
- [Versioning Policy](docs/SCULPT_Versioning.md)

## License
MIT
