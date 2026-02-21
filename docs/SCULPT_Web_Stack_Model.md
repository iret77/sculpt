# SCULPT Web Stack Model

(C) 2026 byte5 GmbH

## Goal
Support modern web delivery reality without hard-wiring SCULPT to one framework.

SCULPT web builds now follow a **two-layer model**:

1. **Standard Web App IR** (framework-agnostic)
2. **Stack Adapter** (framework/runtime specific)

---

## Layer 1: Standard Web App IR

SCULPT + LLM produce a normalized app model (`web-app-ir`) that captures:
- app profile (`standard`, `next-app`, `laravel-mvc`)
- views/pages/forms/modals
- navigation graph
- user actions (query/mutation/http/local)
- data model/validation intents

Schema reference:
- `ir-schemas/web-app-ir.json`

This layer is stable and portable.

---

## Layer 2: Stack Adapters

Adapters map standard IR into concrete stack output.

Current adapter registry shape (from `sculpt target stacks --target web`):
- `builtin.web.standard@1` (frontend)
- `provider.web.next@1` (frontend, external provider)
- `provider.web.laravel@1` (backend-driven, external provider)

Adapters are where framework-specific details belong.
SCULPT language core stays framework-agnostic.

---

## Meta Controls

For web scripts:

```sculpt
@meta target=web
@meta web_profile="next-app"
```

`web_profile` values:
- `standard`
- `next-app`
- `laravel-mvc`

The profile influences generation intent and adapter selection.

---

## Discoverability Commands

- `sculpt target describe --target web`
- `sculpt target packages --target web`
- `sculpt target exports --target web --package builtin.web.ui@1`
- `sculpt target stacks --target web`

---

## Design Rule

SCULPT should describe **application intent** and **constraints**, not framework APIs.
Framework specifics live in adapters and contracts, not in language syntax.
