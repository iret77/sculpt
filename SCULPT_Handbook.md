# SCULPT Handbook (v0.1)

(C) 2026 byte5 GmbH

This handbook explains how to work with SCULPT as a developer:
- what the compiler does,
- how language constructs fit together,
- how to build, freeze, replay, and run programs,
- how to structure projects for team-scale software.

This is a practical guide.  
For strict normative rules, see the dedicated specification documents listed in [README](README.md).

## 1) Mental Model
SCULPT is a convergent language:
- You define intent and constraints.
- Non-deterministic regions (`nd`) define a solution space.
- The compiler produces compact IR and calls an LLM.
- Target providers validate/build deterministic output artifacts.

Think of SCULPT as **code-driven shaping** of solution space, not prose prompting.

## 2) Compiler Pipeline
For `sculpt build`:
1. Parse source code into AST.
2. Run semantic validation.
3. Convert to SCULPT IR.
4. Generate compact LLM IR.
5. Call selected LLM provider.
6. Validate target IR.
7. Run deterministic target build.
8. Write build artifacts and metadata.

### Artifacts
Each script gets an isolated output directory:
- `dist/<script_name>/ir.json`
- `dist/<script_name>/target.ir.json`
- `dist/<script_name>/nondet.report`
- `dist/<script_name>/build.meta.json`

This isolation avoids cross-script collisions and enables clean per-script run/replay behavior.

## 3) Command Guide

### `sculpt examples`
Writes curated examples into `examples/`.

### `sculpt build <file.sculpt> --target <cli|gui|web>`
Runs full compile pipeline and produces artifacts.

### `sculpt run <file.sculpt> [--target ...]`
Runs the latest build output for the selected script.

### `sculpt freeze <file.sculpt> [--target ...]`
Builds and writes `sculpt.lock` to lock deterministic replay input.

### `sculpt replay <file.sculpt> [--target ...]`
Rebuilds using `sculpt.lock` without a fresh LLM generation.

### `sculpt clean <file.sculpt>` / `sculpt clean --all`
Removes script-specific artifacts or the whole `dist/`.

### `sculpt auth check --provider <name> [--verify]`
Checks provider auth configuration, optionally verifies with API call.

### `sculpt gate check <gate.json>`
Evaluates pre-registered release quality gates and returns non-zero on failure.

## 4) Build vs Freeze vs Replay
- **Build**: regular compile path, typically LLM-backed.
- **Freeze**: build + lock deterministic replay input.
- **Replay**: deterministic rebuild from lock, useful for CI and reproducibility.

If you need reproducibility across machines and team members, use freeze + replay.

## 5) Language Guide

### 5.1 Required Root
Every script starts with:

```sculpt
module(App)
  ...
end
```

Dot-qualified module paths are supported for domain namespacing:

```sculpt
module(Billing.Account.Invoice)
```

### 5.2 Core Blocks
- `flow(name)`: state flow graph
- `state(name)`: named state inside a flow
- `state()`: global state storage
- `rule(name, ...)`: deterministic rule block
- `nd(name, ...)`: non-deterministic solution block

### 5.3 State Transitions

```sculpt
start > Title
on key(Enter) > Menu
```

The transition symbol is always `>`.

### 5.4 Rules
Rules use `on ...` or `when ...` triggers and deterministic actions:

```sculpt
rule(tick)
  on tick
    counter += 1
  end
end
```

### 5.5 ND Blocks
ND blocks define candidate generation and hard constraints:

```sculpt
nd(layoutPlan)
  propose layout(type: "rooms")
  satisfy(
    insideBounds(width: 10, height: 5),
    noOverlap(),
    reachablePathExists()
  )
end
```

`propose` expands the space; `satisfy` narrows it.

### 5.6 Comments
SCULPT supports line comments with `#` or `;`.

```sculpt
# This is a comment
; This is also a comment
```

Use comments for intent and constraints, not for repeating obvious code.

## 6) Meta Configuration In Code
Use `@meta` for non-logic compile hints:

```sculpt
@meta target=gui
@meta layout=explicit
@meta strict_scopes=true
```

You can set one key per line, or multiple key-value pairs in one line:

```sculpt
@meta target=gui layout=explicit
```

### 6.1 Supported Meta Keys (Current)
- `target`: default target for this script (`cli`, `gui`, `web`).
- `layout`: currently `explicit` is used for GUI flows that require explicit layout data.
- `strict_scopes`: enables stricter shadowing checks in semantic validation.
- `nd_budget`: convergence budget in range `0..100` (lower means stricter ND tolerance).
- `confidence`: expected convergence confidence in range `0.0..1.0`.
- `max_iterations`: maximum LLM compile retries before fallback.
- `fallback`: fallback policy when LLM compile keeps failing (`fail`, `stub`, `replay`).
- `requires`: comma-separated capability requirements checked against the selected target contract.

### 6.2 How Meta Interacts With CLI Flags
- CLI flags have highest priority (for example `--target` overrides `@meta target=...`).
- If `--target` is omitted, the compiler uses `@meta target` when present.
- If neither is set, build commands fail with a target-required error.
- Unknown `@meta` keys fail contract validation unless declared in the target contract (or prefixed with `x_` for extension keys).

### 6.3 Practical Examples
- Stable script-level target:
  - `@meta target=cli`
- GUI-specific constraint:
  - `@meta target=gui`
  - `@meta layout=explicit`
- Strict validation for critical modules:
  - `@meta strict_scopes=true`
- Convergence controls:
  - `@meta nd_budget=35`
  - `@meta confidence=0.80`

## 7) Providers

### 7.1 LLM Providers
Supported providers:
- `openai`
- `anthropic`
- `gemini`
- `stub`

Provider selection order:
1. CLI flag (`--provider`)
2. `sculpt.config.json` default provider

`--model` overrides provider default model.

### 7.2 Target Providers
Built-in targets:
- `cli`
- `gui`
- `web`

Target providers are responsible for deterministic build execution and run descriptor behavior.

Current `gui` backend by OS:
- macOS: native SwiftUI build via SwiftPM.
- Windows/Linux: Python Tkinter desktop build (initial cross-platform parity path).

## 8) Debugging
Use:

```bash
sculpt build app.sculpt --target cli --debug
```

Levels:
- `compact`
- `raw`
- `all`
- `json`

`--debug` helps inspect IR flow and provider output when build quality is not as expected.

## 9) Team-Scale Conventions
For larger projects:
- Use domain-first module paths (`Billing.*`, `Gameplay.*`, `UI.*`).
- Keep flows focused and avoid monolithic scripts.
- Lock important builds with `freeze`.
- Use replay in CI to verify deterministic regeneration.
- Enforce strict scopes (`@meta strict_scopes=true`) in critical modules.

## 10) Recommended Workflow
1. Start from a small deterministic baseline.
2. Introduce ND blocks only where useful.
3. Add constraints early to keep ND bounded.
4. Build frequently and inspect `nondet.report`.
5. Freeze stable milestones.
6. Replay in CI and before release.

## 11) Where To Go Next
- Syntax reference: [SCULPT Syntax Manifest](SCULPT_Syntax_Manifest.md)
- Semantics reference: [SCULPT Semantics](SCULPT_Semantics.md)
- Target-specific source reference: [SCULPT Target References](SCULPT_Targets_Reference.md)
- Namespace/scopes: [SCULPT Namespaces and Scopes](SCULPT_Namespaces_And_Scopes.md)
- Target architecture: [SCULPT Target Model](SCULPT_Target_Model.md)
- Professional-grade roadmap: [SCULPT Professional-Grade Blueprint](SCULPT_Professional_Grade_Blueprint.md)
- Active backlog: [SCULPT Backlog](SCULPT_Backlog.md)
