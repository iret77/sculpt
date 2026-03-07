# SCULPT Web Target Reference

(C) 2026 byte5 GmbH

## Scope
Target name: `web`  
Standard IR: `web-ir`

Examples:
- `examples/web/web_profile_standard.sculpt` (`@meta web_profile="standard"`)
- `examples/web/incident_status_dashboard.sculpt` (`@meta web_profile="next-app"`)
- `examples/web/support_ticket_board.sculpt` (`@meta web_profile="laravel-mvc"`)

## What Works Today
- Browser-based text rendering
- Keyboard-driven transitions
- Per-item inline CSS object support in target IR

## Provider Packages
Inspect live package metadata with:
- `sculpt target packages --target web`
- `sculpt target exports --target web --package builtin.web.ui@1`
- `sculpt target exports --target web --package builtin.web.input@1`
- `sculpt target exports --target web --package builtin.web.data@1`
- `sculpt target exports --target web --package builtin.web.net@1`
- `sculpt target exports --target web --package builtin.web.guide@1`

Current built-in namespaces:
- `ui.*` from `builtin.web.ui@1`
- `input.*` from `builtin.web.input@1`
- `data.*` from `builtin.web.data@1`
- `net.*` from `builtin.web.net@1`
- `guide.*` from `builtin.web.guide@1` (ND constraints for `satisfy(...)`)

Key exports:
- `ui`: app UI building blocks (`heading`, `panel`, `card`, `tabs`, `modal`, `toast`, `metric`, `chart`, ...)
- `input`: UI events (`key`, `click`, `submit`, `change`, `focus`, `blur`, `navigate`, ...)
- `data`: query primitives (`query`, `filter`, `sort`, `paginate`, `group`, `aggregate`, `join`)
- `net`: API primitives (`get`, `post`, `put`, `patch`, `delete`, `upload`, `download`)
- `guide`: ND constraints (`noOverlap`, `responsiveBreakpoints`, `accessibleColorContrast`, ...)

## Supported Render Calls
Practical SCULPT pattern:
- `ui.text("...", color: "...")`

Effective item kind in `web-ir`:
- `kind: "text"`

`button` is not part of current `web-ir`.

## Supported Events
Runtime dispatch uses:
- `input.key(<normalized_key>)`

Useful keys:
- `input.key(enter)`
- `input.key(esc)`
- `input.key(space)`
- alphanumeric keys lowercased

## Flow Behavior
- `start > <State>` required.
- `on input.key(...) > <State>` transitions are applied in browser runtime.
- Web runtime listens to `window.keydown`.

## Target Meta/Contract Notes
- Typical:
  - `@meta target=web`
- Optional profile selection:
  - `@meta web_profile=standard`
  - `@meta web_profile="next-app"`
  - `@meta web_profile="laravel-mvc"`
- `layout=explicit` is not valid for Web in the current built-in contract.
- Capability requirements can be declared via:
  - `@meta requires="runtime.web,render.text"`

## Stack Adapters
Inspect available stack adapters:

```bash
sculpt target stacks --target web
```

The built-in web emitter is `builtin.web.standard@1`.  
`next-app` and `laravel-mvc` are modeled as adapter profiles and can be implemented as external target providers.

## Known Limits (Current)
- No built-in web component system in this target yet
- Runtime behavior parity across all advanced widgets is still evolving
- Styling depth still depends on target IR extensions and providers

## ND Constraints (Strict Mode)
- In `satisfy(...)`, use:
  - `guide.*(...)` (contract constraint),
  - `?name(...)` (soft define),
  - `?"..."` (inline ND prompt).

## Minimal Example
```sculpt
@meta target=web
module(App.WebDemo)
  flow(Main)
    start > Title
    state(Title)
      ui.text("Web Demo", color: "blue")
      ui.text("Press Enter", color: "green")
      on input.key(enter) > Done
      on input.key(esc) > Exit
    end
    state(Done)
      ui.text("Done", color: "black")
      terminate
    end
    state(Exit)
      terminate
    end
  end
end
```
