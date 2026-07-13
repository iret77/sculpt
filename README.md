# SCULPT

## Elevator Pitch
**SCULPT is the evidence-carrying change compiler for AI-written software.**

Developers express the boundaries, invariants, freedoms, and proof required for
a change as typed source. SCULPT lets bounded compiler agents patch an existing
system, then refuses acceptance until every affected critical obligation has
fresh, policy-approved evidence.

SCULPT is not a better prompt and not a general app generator. Its product
hypothesis is a programmable assurance boundary between AI-generated change and
production software.

[Read the Convergent Programming concept](docs/SCULPT_Convergent_Programming_Concept.md).

## What This Repository Contains
- Rust CLI compiler (`sculpt`)
- Interactive TUI mode
- LLM provider integration (`openai`, `anthropic`, `gemini`, `stub`)
- Built-in target providers (`cli`, `gui`, `web`)
- Example SCULPT programs

## Why SCULPT
Modern coding agents already plan, edit, build, test, and repair from persistent
repository context and structured specifications. SCULPT must add what those
workflows do not enforce as one typed system:

- closed-world change boundaries and decision permissions,
- stable obligations linked to exact semantic subjects,
- automatic evidence invalidation after relevant changes,
- semantic impact analysis and selective regeneration,
- patches, decisions, provenance, and evidence as one acceptance unit.

## Status
- The repository contains a Rust compiler prototype, CLI/TUI, language
  front-end, contracts, freeze/replay experiments, tests, and built-in
  `cli`, `gui`, and `web` providers.
- The evidence-carrying Brownfield change compiler described by the concept is
  **not implemented yet**.
- Existing targets and showcase benchmarks are prototype artifacts and
  maintenance-only; they are not proof of the new thesis.
- The project now follows a bounded existence program with immutable Full-Go
  or archive gates.

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
  - [Current Evidence Status](poc/SCULPT_Evidence_Status.md)
  - [SCULPT Case Studies Overview](poc/SCULPT_Case_Studies_Overview.md)
  - [SCULPT Roadmap](docs/SCULPT_Roadmap.md)

## Documentation Map
Documentation has been moved under `/docs` for easier navigation:
- [Docs Index](docs/README.md)
- [Versioning Policy](docs/SCULPT_Versioning.md)

## License
MIT
