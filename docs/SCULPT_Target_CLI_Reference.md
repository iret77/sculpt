# SCULPT CLI Target Reference

(C) 2026 byte5 GmbH

## Scope
Target name: `cli`  
Standard IR: `cli-ir`

## What Works Today
- Text rendering in terminal
- State transitions via keyboard events
- Colorized text output
- Runtime rules for `on` and `when` (including `and`/`or`, `!=`)

## Provider Packages
Inspect live package metadata with:
- `sculpt target packages --target cli`
- `sculpt target exports --target cli --package builtin.cli.ui@1`
- `sculpt target exports --target cli --package builtin.cli.input@1`

Current built-in namespaces:
- `ui.*` from `builtin.cli.ui@1`
- `input.*` from `builtin.cli.input@1`

## Supported UI Calls
Recommended (explicit import + namespace):
- `ui.text("...", color: "...")`

Legacy shorthand (still accepted during migration):
- `ui.text("...", color: "...")`

Effective item kind in `cli-ir`:
- `kind: "text"`

`button` is not supported in `cli-ir`.

## Supported Events
Runtime dispatch uses:
- `input.key(<normalized_key>)`

Useful keys:
- `input.key(enter)`
- `input.key(esc)`
- `input.key(space)`
- `input.key(a)`, `input.key(1)`, etc. (lowercased)

## Flow Behavior
- `start > <State>` is required.
- `on input.key(...) > <State>` transitions are used at runtime.
- `run` and `terminate` are language-level and validated before build.
- State-local rules are supported and scoped to their state.

## Target Meta/Contract Notes
- Typical:
  - `@meta target=cli`
- `layout=explicit` is not valid for CLI.
- Capability requirements can be declared via:
  - `@meta requires="capability.a,capability.b"`

## Known Limits (Current)
- Terminal-only UI
- No native button widgets
- No complex layout primitives
- `when` runtime currently supports scalar comparisons only

## Minimal Example
```sculpt
@meta target=cli
module(App.CliDemo):
  use(cli.ui)
  use(cli.input, as: input)
  flow(Main):
    start > Title
    state(Title):
      ui.text("CLI Demo", color: "yellow")
      ui.text("Enter = Continue", color: "blue")
      on input.key(enter) > Done
      on input.key(esc) > Exit
    end
    state(Done):
      ui.text("Done", color: "green")
      terminate
    end
    state(Exit):
      terminate
    end
  end
end
```
