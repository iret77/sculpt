# SCULPT Syntax Manifest v0.2 (Draft)

(C) 2026 byte5 GmbH

## 1) Goal
SCULPT is a **convergent** language:
more code narrows the solution space and increases predictability, while remaining non-fully-deterministic by design.

**Design principles**
- **Not whitespace-sensitive.** Whitespace improves readability, never syntax.
- **Structural clarity through symbols.** Code should be visually structured without noisy syntax.
- **Uniform block shape.** Core constructs follow one predictable pattern.
- **Scalable for AI pipelines.** Designed for current LLM limits without blocking future growth.

## 2) Block Form
**Every block uses a function-style signature.**  
Whitespace is for readability only and is **never** part of syntax.

**Meta header (optional, non-logic):**
```
@meta target=gui layout=explicit
@meta author="team"
```

**Note (GUI action semantics):**
For a simple OK modal action, the compiler uses:
`action="modal.ok"`

```
<blockType>(<name, params...>)
  ...
end
```

Examples:
```
module(App)
flow(Game)
state(Title)
state()
rule(tick)
nd(chooseLayout, level)
```

Benefit: consistent and logical form, language-independent.

**Note (namespace path):**
`module(...)` may be dot-qualified, e.g.:
`module(Billing.Account.Invoice)`

## 3) Transition Syntax
**Transitions use one symbol:** `>`

```
start > Title
on key(Enter) > Play
```

`>` is compact, easy to type, and visually clear.

## 4) Primary Block Types
- `module(name)` -> root block (required, exactly one per file)
- `flow(name)` -> state flow
- `state(name)` -> named state
- `state()` -> global state block (unnamed)
- `rule(name)` -> deterministic rule block
- `nd(name, ...)` -> non-deterministic solution block

## 5) Statements Inside `state(...)`
- **Render calls** (meaning comes from the selected target contract):
  ```
  render text("Hello", color: "yellow")
  ```
- **Transition:**
  ```
  on key(Enter) > Play
  ```
- **Run flow:**
  ```
  run Loop
  ```
- **Terminate:**
  ```
  terminate
  ```

## 6) Rule Syntax
```
rule(tick)
  on tick
    counter += 1
  end
end
```

or

```
rule(finish)
  when counter >= 3
    emit done
  end
end
```

## 7) ND Syntax
```
nd(chooseLayout, level)
  propose layout(type: "rooms")
  satisfy(
    insideBounds(width: 10, height: 5),
    noOverlap(),
    reachablePathExists()
  )
end
```

## 8) Expressions (Current)
- Literals: numbers, strings, null
- Identifiers: `counter`
- Calls: `key(Enter)`
- Assignment: `=`, `+=`
- Compare: `>=`

## 9) Visual Rhythm (Example)
```
module(App)
  flow(Main)
    start > Title

    state(Title)
      render text("HELLO", color: "yellow")
      on key(Enter) > Play
    end

    state(Play)
      run Loop
      on done > Title
    end
  end

  state()
    counter = 0
  end
end
```

## 10) Comments (Non-Syntax)
Comments start with `#` or `;` and may contain arbitrary text.

```
# UI
; Logic
```

## Fixed Decisions
1. Block form is mandatory: `block(name, params...)`
2. Transition symbol is `>`
3. Global state is represented by `state()`
4. Multiple ND parameters are supported: `nd(name, param1, param2)`
