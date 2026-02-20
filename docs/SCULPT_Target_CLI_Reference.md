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

## Supported Render Calls
From SCULPT source, the practical pattern is:
- `render text("...", color: "...")`

Effective item kind in `cli-ir`:
- `kind: "text"`

`button` is not supported in `cli-ir`.

## Supported Events
Runtime dispatch uses:
- `key(<normalized_key>)`

Useful keys:
- `key(enter)`
- `key(esc)`
- `key(space)`
- `key(a)`, `key(1)`, etc. (lowercased)

## Flow Behavior
- `start > <State>` is required.
- `on key(...) > <State>` transitions are used at runtime.
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
module(App.CliDemo)
  flow(Main)
    start > Title
    state(Title)
      render text("CLI Demo", color: "yellow")
      render text("Enter = Continue", color: "blue")
      on key(enter) > Done
      on key(esc) > Exit
    end
    state(Done)
      render text("Done", color: "green")
      terminate
    end
    state(Exit)
      terminate
    end
  end
end
```
