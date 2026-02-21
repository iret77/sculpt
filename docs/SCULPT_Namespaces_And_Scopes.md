# SCULPT Namespaces & Scopes (Language 1.0)

(C) 2026 byte5 GmbH

## Purpose
Define the first professional-grade naming and scope model for SCULPT, optimized for:
- multi-team development,
- large codebases,
- AI-first incremental compilation.

This model is intentionally lightweight and DDD-aligned.

## 1) Namespace Model (DDD-Aligned)

SCULPT modules are organized by domain path, not by technical folder names.

Canonical form:
```sculpt
module(Billing.Account.Invoice)
```

Rules:
- `module(...)` MAY be a dot-qualified path.
- Each segment MUST be a valid identifier.
- Namespace root SHOULD map to a bounded context (`Billing`, `Inventory`, `Gameplay`, etc.).
- Contracts across namespace roots MUST be explicit (`import`/`contract`, future phase).

Why:
- Natural fit for team ownership.
- Smaller AI context per bounded domain.

## 2) Symbol Scope Levels

Resolution order for unqualified names:
1. Local block scope (future local vars/params)
2. Rule scope
3. State scope
4. Flow scope
5. Module scope
6. Imported namespaces (future phase)

Core rules:
- A symbol MUST be declared before use in the same scope level.
- Inner scopes MAY shadow outer scopes only when explicitly allowed by policy.
- Shadowing SHOULD be disabled by default in strict mode.

## 3) Name Uniqueness Rules

Within one module:
- `flow(name)` names MUST be unique.
- `rule(name)` names MUST be unique.

Within one flow:
- `state(name)` names MUST be unique.
- Event handler signature (`on <eventCall>`) in a state MUST be unique.

Global state:
- Multiple `state()` blocks MAY exist.
- Merged global symbols MUST NOT conflict.

## 4) Fully Qualified Names (FQN)

FQN format:
`<module_path>.<flow>.<state>.<symbol>`

Examples:
- `Billing.Account.Invoice.MainFlow.Draft.totalGross`
- `Gameplay.Snake.Game.Play.score`

Semantics:
- Diagnostics SHOULD report both short and fully qualified names.
- IR SHOULD store canonical FQNs for stable diff/replay behavior.

## 5) Minimal Syntax Additions (Language 1.0)

No heavy new grammar is required for language 1.0.

Enabled now:
- Dot-qualified module names: `module(A.B.C)`

Reserved for next phase:
- `import(...)`
- `export(...)`
- `alias(...)`

## 6) Diagnostics (New Error Codes)

Namespace/Scope errors:
- `NS501` Invalid namespace segment.
- `NS502` Duplicate namespace symbol in same scope.
- `NS503` Unknown qualified name reference.
- `NS504` Illegal cross-namespace reference without contract/import.
- `NS505` Forbidden shadowing in strict mode.
- `NS506` Ambiguous unqualified symbol reference.

## 7) Example (Team-Scale Structure)

```sculpt
# Billing context
module(Billing.Account.Invoice)

  flow(Main)
    start > Draft

    state(Draft)
      on key(Enter) > Validate
    end

    state(Validate)
      run RiskCheck
      on done > Finalize
    end

    state(Finalize)
      terminate
    end
  end

  flow(RiskCheck)
    start > Check
    state(Check)
      # future: explicit contract call to Fraud namespace
      emit done
    end
  end

  state()
    invoiceTotal = 0
    currency = "EUR"
  end
end
```

## 8) AI-First Impact

This model directly improves AI compile quality:
- Smaller compile units by namespace,
- clearer symbol resolution,
- lower ambiguity in prompts/compact IR,
- better cache/replay granularity.

## 9) Implementation Steps

1. Parser: allow dot-qualified identifiers in `module(...)`.
2. Semantic validator: enforce `NS501..NS506`.
3. IR: add canonical module path + FQNs.
4. CLI/TUI diagnostics: render scoped errors with FQN context.
5. Incremental compile planner: recompile only changed namespace units + dependents.
