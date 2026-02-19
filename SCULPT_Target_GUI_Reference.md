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
- `render text("...", color: "...")`
- `render button("...", action: "modal.ok")` (LLM/IR mapping dependent)

Effective item kinds in `gui-ir`:
- `kind: "text"`
- `kind: "button"`

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
      render text("Native GUI Demo", color: "blue")
      render text("Click the button", color: "secondary")
      render button("Open OK", action: "modal.ok")
      terminate
    end
  end
end
```
