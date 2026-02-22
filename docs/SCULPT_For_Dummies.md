# SCULPT For Dummies

(C) 2026 byte5 GmbH

This guide is for developers who want to go from zero to “I can write my own SCULPT script”.

## 1) SCULPT in Plain Words

SCULPT is a programming language where you describe:
- what your app should do,
- how screens/states connect,
- and which rules must always be true.

Then the compiler does two extra jobs for you:
- it asks an LLM for target code planning,
- and it builds deterministic output through target providers.

### Three words you need

- `IR` (Intermediate Representation):
  - a machine blueprint between your SCULPT code and final app output.
- `ND` (Non-Deterministic):
  - you allow multiple valid solutions.
- `Constraint`:
  - a hard rule that limits ND (“must be readable”, “no overlap”, etc.).

## 2) The 5 Core Blocks You Actually Use

| Block | What it means (ELI5) | Tiny example |
|---|---|---|
| `module(...)` | Your file’s main container/name. | `module(HelloWorld): ... end` |
| `flow(...)` | The route map of your app. | `start > Menu` |
| `state(...)` | One screen/mode/step. | `state(Menu): ... end` |
| `rule(...)` | Automatic logic that runs on trigger. | `when lives < 1: emit done` |
| `nd(...)` | “Compiler, solve this with AI, but follow these limits.” | `satisfy(noOverlap())` |

## 3) First Program You Can Run

```sculpt
@meta target=cli

module(HelloWorld):
  use(cli.ui)
  use(cli.input) as input

  flow(App):
    start > Show

    state(Show):
      ui.text("Hallo", color: "yellow")
      ui.text("Welt", color: "blue")
      on input.key(Esc) > Exit
    end

    state(Exit):
      terminate
    end
  end
end
```

Run it:

```bash
sculpt build hello_world.sculpt --provider stub
sculpt run hello_world.sculpt
```

## 4) How to Think While Writing SCULPT

Use this loop:
1. Build the **flow** first (`flow`, `state`, transitions).
2. Add **state data** (`state(): ... end`) for variables.
3. Add **rules** for predictable behavior.
4. Add **ND blocks** only where flexibility helps.
5. Build and run often.
6. Freeze when result is stable.

## 5) State Data vs Rules (Important)

### Global data

```sculpt
state():
  counter = 0
  speedMs = 120
end
```

### Deterministic logic

```sculpt
rule(tick):
  on input.tick:
    counter += 1
  end
end
```

Think:
- `state()` = memory
- `rule()` = automatic behavior

## 6) ND Without Chaos

Bad ND:

```sculpt
nd(layout):
  propose dashboard()
end
```

Better ND:

```sculpt
nd(layout):
  propose dashboard(kind: "ops")
  satisfy(
    noOverlap(),
    highContrast(),
    keyboardNavigable()
  )
end
```

Rule of thumb:
- More `satisfy(...)` constraints = more stable output.

## 7) Where `ui.text` and `input.key` Come From

They are not built-in language keywords.  
They come from provider namespaces you import via `use(...)`.

Example:

```sculpt
use(cli.ui)
use(cli.input) as input
```

Now `ui.*` and `input.*` are available in this module.

## 8) Multi-File Projects (When One File Is Not Enough)

If you use `import(...)`, you must build in project mode (`.sculpt.json`).

### Create project file automatically

```bash
sculpt project create billing -p examples/business -f "*.sculpt"
```

This creates `billing.sculpt.json`.

### Build project

```bash
sculpt build examples/business/billing.sculpt.json --provider stub
```

### Import from another module

```sculpt
module(Billing.App):
  import(Billing.Shared.InvoiceRules) as Shared
  use(cli.ui)
end
```

## 9) Meta Keys You Will Really Use

| Meta key | Why you use it |
|---|---|
| `@meta target=cli|gui|web` | Default target for this script/project. |
| `@meta nd_budget=...` | How much ND freedom you allow. |
| `@meta confidence=...` | Expected convergence confidence. |
| `@meta fallback=fail|stub|replay` | What to do if LLM compile fails repeatedly. |
| `@meta max_iterations=...` | Retry limit for convergence loop. |

## 10) Common Errors (Fast Fixes)

| Error | Meaning | Fix |
|---|---|---|
| `Target required` | No target in meta and no `--target`. | Set `@meta target=...` or pass `--target`. |
| `Imports require a project file` | You used `import(...)` in standalone `.sculpt`. | Build a `.sculpt.json` project file and compile that. |
| Semantic validation failed | Namespaces/rules/meta inconsistent. | Read diagnostics, fix exact line, rebuild. |

## 11) Practical Command Set

```bash
sculpt examples
sculpt build app.sculpt --provider stub
sculpt run app.sculpt
sculpt freeze app.sculpt --provider stub
sculpt replay app.sculpt
sculpt clean app.sculpt
```

Project mode:

```bash
sculpt project create myproj -p . -f "*.sculpt"
sculpt build myproj.sculpt.json --provider stub
sculpt run myproj.sculpt.json
```

## 12) Final Checklist Before You Build

- `module(...)` exists and is valid.
- `flow` has a clear `start > ...`.
- every transition target state exists.
- `use(...)` imports required provider namespaces.
- ND blocks have useful `satisfy(...)` constraints.
- target is set by `@meta target=...` or CLI flag.

## 13) Next Docs

- Fast setup: [SCULPT Quick Start](SCULPT_Quick_Start.md)
- Full behavior: [SCULPT Handbook](SCULPT_Handbook.md)
- Exact syntax: [SCULPT Syntax Manifest](SCULPT_Syntax_Manifest.md)
- Target APIs: [SCULPT Target References](SCULPT_Targets_Reference.md)
