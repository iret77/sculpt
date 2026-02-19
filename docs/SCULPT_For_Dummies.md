# SCULPT For Dummies

(C) 2026 byte5 GmbH

This guide is not a full reference.  
It is a practical "how do I get useful results fast?" guide for developers.

## 1) SCULPT in 60 Seconds
- You do not write every implementation detail.
- You write intent, flow, and constraints.
- SCULPT compiles that into target IR with an LLM.
- Target providers build a stable, runnable result.

Think: less "how to code each line", more "what this software must do and how strict it should be."

## 2) When SCULPT Is a Good Fit
Use SCULPT when:
- You build workflow-heavy features.
- You want repeatable AI-assisted output.
- You want clear constraints instead of long prose prompts.

Do not start with SCULPT when:
- You need very low-level, exact step-by-step control from line 1.
- You already have a stable, mature codebase for the exact same task.

## 3) Mental Workflow
Use this loop every time:
1. Define the module and flow.
2. Add only essential states and transitions.
3. Add rules for clear, predictable updates.
4. Add ND blocks only where needed.
5. Add constraints early.
6. Build, run, inspect, refine.
7. Freeze when stable.

`ND` means: you do not force one exact implementation.
You define what must be true, and SCULPT+LLM find a valid solution within those limits.

## 4) Minimal Building Blocks
- `module(...)`: root container.
- `flow(...)`: app flow (screens/states).
- `state(...)`: one step/screen.
- `rule(...)`: clear, predictable logic.
- `nd(...)`: controlled non-determinism.
- `@meta`: compile-time controls (`target`, convergence, strictness).

## 5) Copy-Paste Starter Patterns

### Pattern A: Deterministic Hello World
```sculpt
@meta target=cli
module(HelloWorld)
  flow(App)
    start > Show
    state(Show)
      render text("Hello", color: "yellow")
      terminate
    end
  end
end
```

### Pattern B: Workflow Screen to Screen
```sculpt
@meta target=gui
module(SimpleFlow)
  flow(App)
    start > Start
    state(Start)
      render text("Press Enter", color: "blue")
      on key(Enter) > Done
    end
    state(Done)
      render text("Completed", color: "green")
      terminate
    end
  end
end
```

### Pattern C: Rule-Based Update
```sculpt
module(Counter)
  state()
    value = 0
  end

  rule(tickRule)
    on tick
      value += 1
    end
  end
end
```

### Pattern D: ND with Tight Constraints
```sculpt
@meta nd_budget=25
@meta confidence=0.9
module(LayoutPlan)
  nd(layout)
    propose layout(kind: "dashboard")
    satisfy(
      hasHeader(),
      hasPrimaryAction(),
      noOverlap()
    )
  end
end
```

### Pattern E: Convergence Controls + Fallback
```sculpt
@meta target=gui
@meta max_iterations=3
@meta fallback=replay
module(StableGui)
  flow(App)
    start > Main
    state(Main)
      render text("Stable path", color: "white")
      terminate
    end
  end
end
```

## 6) Commands You Actually Need
```bash
sculpt examples
sculpt build examples/hello_world.sculpt --target cli
sculpt run examples/hello_world.sculpt --target cli
sculpt freeze examples/hello_world.sculpt --target cli
sculpt replay examples/hello_world.sculpt --target cli
sculpt gate check poc/gates/incident_triage_vibe_gate.json
```

## 7) Common Mistakes (and Fixes)
- Mistake: no target selected.
  - Fix: set `@meta target=...` or pass `--target`.
- Mistake: ND too open.
  - Fix: add stronger `satisfy(...)` constraints and reduce `nd_budget`.
- Mistake: unstable results.
  - Fix: set `max_iterations`, `fallback`, and use `freeze/replay`.
- Mistake: unexpected build rejection.
  - Fix: check contract/meta errors (`C901..C904`), align script meta with target contract.

## 8) Team Usage (Plain Version)
- Split by domain modules (`Billing.*`, `Ops.*`, `UI.*`).
- Keep flows small.
- Use strict scopes in critical modules.
- Use `freeze` for milestones and `replay` in CI.
- Gate major claims/results with `sculpt gate check`.

## 9) Scaling from Small to Large
- Start with one script proving behavior.
- Split into multiple module files by domain.
- Move shared conventions to team standards (meta, constraints, naming).
- Add contracts and gate files early to prevent drift.

## 10) Where To Go Next
- If you want quick setup: [SCULPT Quick Start](SCULPT_Quick_Start.md)
- If you want full usage details: [SCULPT Handbook](SCULPT_Handbook.md)
- If you want target-specific functions/events: [SCULPT Target References](SCULPT_Targets_Reference.md)
- If you need exact language rules: [SCULPT Syntax Manifest](SCULPT_Syntax_Manifest.md)
- If you need semantic rules and diagnostics: [SCULPT Semantics](SCULPT_Semantics.md)
