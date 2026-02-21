# SCULPT Examples

(C) 2026 byte5 GmbH

Examples are grouped by intent so new developers can find the right starting point quickly.

All current examples use explicit provider namespaces via:
- `use(<target>.ui)`
- `use(<target>.input) as input`
Legacy shorthand (`render ...`, `key(...)`) is rejected by default.

## 1) Getting Started
- `getting-started/hello_world.sculpt`
  - Smallest deterministic program.
- `getting-started/native_window.sculpt`
  - Basic GUI output with explicit target metadata.

## 2) Games
- `games/breakout_high_nd.sculpt`
  - Compact high-ND Breakout that delegates mechanics/UI detail via constraints.
- `games/snake_high_nd.sculpt`
  - Playable and visually richer Snake with high ND (creative runtime freedom).
- `games/snake_low_nd.sculpt`
  - Playable and visually richer Snake with low ND (explicit mechanics and scoring).
- `games/breakout_low_nd.sculpt`
  - Detailed low-ND Breakout with explicit flow and rule structure.

## 3) Business
- `business/invoice_review.sculpt`
  - Simple approval/rejection flow.
- `business/incident_triage_assistant.sculpt`
  - Operational playbook assistant.
- `business/expense_approval_workflow.sculpt`
  - Realistic workflow-oriented example with strict logic and light ND.
- `business/modular_invoice_app.sculpt`
  - Multi-file business app using namespace imports (`import(Billing.Shared.InvoiceRules)`) and project file `business/modular_invoice_app.sculpt.json`.

## 4) Web
- `web/incident_status_dashboard.sculpt`
  - Web-target incident overview with clear state navigation (`web_profile=next-app`).
- `web/support_ticket_board.sculpt`
  - Small service-desk use case with ticket detail flow and SLA screen (`web_profile=laravel-mvc`).

## Quick Run
```bash
sculpt build examples/getting-started/hello_world.sculpt --target cli
sculpt run examples/getting-started/hello_world.sculpt --target cli
```
