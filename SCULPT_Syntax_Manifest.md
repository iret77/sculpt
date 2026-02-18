# SCULPT Syntax‑Manifest v0.2 (Entwurf)

## 1) Ziel
SCULPT ist eine **konvergente** Sprache:
Mehr Code → kleinere Lösungsmenge → deterministischer, aber nie vollständig deterministisch.

**Leitprinzipien**
- **Nicht‑Whitespace‑sensitiv.** Whitespaces dienen der Lesbarkeit, niemals der Syntax.
- **Strukturelle Klarheit durch Symbole.** Syntax soll visuell gliedern, ohne zu „schreien“.
- **Einheitlichkeit:** Gleicher Aufbau für alle Blocktypen.

## 2) Block‑Form
**Jeder Block ist eine Funktionssignatur.**  
Whitespace dient nur der Lesbarkeit und ist **niemals** Syntax.

```
<blockType>(<name, params...>)
  ...
end
```

Beispiele:
```
module(App)
flow(Game)
state(Title)
state()
rule(tick)
nd(chooseLayout, level)
```

Vorteil: konsequente, logische Form, unabhängig von Sprache.

## 3) Transition‑Syntax
**Transitions verwenden ein einziges Symbol:** `>`

```
start > Title
on key(Enter) > Play
```

`>` ist minimal, leicht zu tippen (Shift+.) und visuell eindeutig.

## 4) Primäre Blocktypen
- `module(name)` → Root‑Block (pflichtig, genau 1 pro Datei)
- `flow(name)` → Zustands‑Ablauf
- `state(name)` → Zustand
- `state()` → globaler Zustand (ohne Namen)
- `rule(name)` → deterministische Regeln
- `nd(name, ...)` → nicht‑deterministische Lösungsräume

## 5) Statements im State
- **Render‑Calls** (Bedeutung kommt aus dem Target‑Vertrag):
  ```
  render text("Hello", color: "yellow")
  ```
- **Transition:**
  ```
  on key(Enter) > Play
  ```
- **Run Flow:**
  ```
  run Loop
  ```
- **Terminate:**
  ```
  terminate
  ```

## 6) Rule‑Syntax
```
rule(tick)
  on tick
    counter += 1
  end
end
```

oder

```
rule(finish)
  when counter >= 3
    emit done
  end
end
```

## 7) ND‑Syntax
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

## 8) Expressions (MVP)
- Literale: numbers, strings, null
- Identifiers: `counter`
- Calls: `key(Enter)`
- Assignment: `=`, `+=`
- Compare: `>=`

## 9) Visual Rhythm (Beispiel)
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

## 10) Kommentare (non‑syntax)
Kommentare beginnen mit `#` oder `;` und können beliebigen Text enthalten.

```
# UI
; Logic
```

## Entscheidungen (festgelegt)
1. Block‑Form ist verbindlich: `block(name, params...)`
2. Transition‑Symbol ist `>`
3. Global State wird über `state()` definiert
4. Mehrere ND‑Parameter: `nd(name, param1, param2)`
