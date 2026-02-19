# SCULPT Web Target Reference

(C) 2026 byte5 GmbH

## Scope
Target name: `web`  
Standard IR: `web-ir`

## What Works Today
- Browser-based text rendering
- Keyboard-driven transitions
- Per-item inline CSS object support in target IR

## Supported Render Calls
Practical SCULPT pattern:
- `render text("...", color: "...")`

Effective item kind in `web-ir`:
- `kind: "text"`

`button` is not part of current `web-ir`.

## Supported Events
Runtime dispatch uses:
- `key(<normalized_key>)`

Useful keys:
- `key(enter)`
- `key(esc)`
- `key(space)`
- alphanumeric keys lowercased

## Flow Behavior
- `start > <State>` required.
- `on key(...) > <State>` transitions are applied in browser runtime.
- Web runtime listens to `window.keydown`.

## Target Meta/Contract Notes
- Typical:
  - `@meta target=web`
- `layout=explicit` is not valid for Web in the current built-in contract.
- Capability requirements can be declared via:
  - `@meta requires="runtime.web,render.text"`

## Known Limits (Current)
- No built-in web component system in this target yet
- No built-in button/action primitives in current schema
- Styling is basic unless target IR extensions are used

## Minimal Example
```sculpt
@meta target=web
module(App.WebDemo)
  flow(Main)
    start > Title
    state(Title)
      render text("Web Demo", color: "blue")
      render text("Press Enter", color: "green")
      on key(enter) > Done
      on key(esc) > Exit
    end
    state(Done)
      render text("Done", color: "black")
      terminate
    end
    state(Exit)
      terminate
    end
  end
end
```
