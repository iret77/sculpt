# SCULPT GUI Target Reference

(C) 2026 byte5 GmbH

## Scope
Target name: `gui`  
Standard IR: `gui-ir`

## Runtime Backends (Current)
- macOS: SwiftUI (SwiftPM build)
- Windows/Linux: Python Tkinter

## What Works Today
- Text rendering in a native desktop window
- Optional button rendering
- Optional `modal.ok` button behavior
- Window title/size from target IR
- Optional explicit layout metadata (`padding`, `spacing`, `align`, `background`)

## Supported Render Calls
Practical SCULPT patterns:
- `ui.text("...", color: "...")`
- `ui.button("...", action: "modal.ok")` (LLM/IR mapping dependent)

Effective item kinds in `gui-ir`:
- `kind: "text"`
- `kind: "button"`

## Provider Packages
Inspect live package metadata with:
- `sculpt target packages --target gui`
- `sculpt target exports --target gui --package builtin.gui.ui@1`
- `sculpt target exports --target gui --package builtin.gui.input@1`
- `sculpt target exports --target gui --package builtin.gui.data@1`
- `sculpt target exports --target gui --package builtin.gui.window@1`
- `sculpt target exports --target gui --package builtin.gui.guide@1`

Current built-in namespaces:
- `ui.*` from `builtin.gui.ui@1`
- `input.*` from `builtin.gui.input@1`
- `data.*` from `builtin.gui.data@1`
- `window.*` from `builtin.gui.window@1`
- `guide.*` from `builtin.gui.guide@1` (ND constraints for `satisfy(...)`)

Key exports:
- `ui`: text/form/layout primitives (`text`, `button`, `input`, `select`, `checkbox`, `table`, `tabs`, `progress`, ...)
- `input`: interaction events (`key`, `click`, `submit`, `change`, `focus`, `blur`, `closeWindow`)
- `data`: deterministic batch/data functions (`csvRead`, `rowCount`, `sortBy`, `writeJson`, `writeCsv`, ...)
- `window`: shell controls (`open`, `close`, `resize`, `modalOk`, `modalConfirm`, `notify`)
- `guide`: ND constraints (`desktopNativeLook`, `focusOrderStable`, `dialogCopyClarity`, ...)

## Layout Support
`gui` supports explicit layout mode:
- `@meta layout=explicit`

Relevant layout fields:
- `padding`
- `spacing`
- `align` (`leading|center|trailing`)
- `background` (`window|grouped|clear`)

## Target Meta/Contract Notes
- Typical:
  - `@meta target=gui`
  - `@meta layout=explicit` (optional)
- Capability requirements can be declared via:
  - `@meta requires="layout.explicit,ui.modal.ok"`

## ND Constraints (Strict Mode)
- In `satisfy(...)`, use:
  - `guide.*(...)` (contract constraint),
  - `?name(...)` (soft define),
  - `?"..."` (inline ND prompt).

## Important Current Limitation
Current GUI runtime generation is focused on the start view and static UI output.
Full event-driven GUI state-machine parity with CLI/Web is still evolving.

Use GUI target today for:
- simple native windows,
- static/low-interaction desktop demos,
- layout and style experiments.

## Minimal Example
```sculpt
@meta target=gui
@meta layout=explicit
module(App.GuiDemo)
  flow(Main)
    start > Home
    state(Home)
      ui.text("Native GUI Demo", color: "blue")
      ui.text("Click the button", color: "secondary")
      ui.button("Open OK", action: "modal.ok")
      terminate
    end
  end
end
```
